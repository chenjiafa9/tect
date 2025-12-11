//! 完全独立于 RightMouseAction 的左键交互系统
//! 左键 = 砍树 / 放置物品（由工具 + 快捷栏决定）
//! 右键只负责相机和移动，不参与任何交互
use bevy::ecs::message::signal_message_update_system;
// use bevy::ecs::event::event_update_system;
use bevy::pbr::wireframe::{Wireframe, WireframeConfig, WireframePlugin};
use bevy::pbr::StandardMaterial; // 新增：用于半透明预览
use bevy::picking::backend::PointerHits;
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::picking::pointer::PointerId;
use bevy::picking::pointer::PointerInteraction;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use tect_assetload::asset_load::GameAssets;
use tect_state::app_state::AppState;

/// 当前手持工具
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerTool {
    None,
    #[default]
    Axe,
    Hammer,
    Shovel,
    // 继续扩展...
}

/// 当前快捷栏选中的可放置物（由 UI 设置）
#[derive(Resource, Default, Clone)]
pub struct SelectedPlaceable {
    pub scene: Handle<Scene>,
    pub name: String,
}

impl SelectedPlaceable {
    pub fn is_valid(&self) -> bool {
        self.scene.is_strong()
    }
}

/// 可被左键破坏的物体（树、石头、箱子等）
#[derive(Component, Debug, Default)]
pub struct Destructible;

/// 已放置的物体（可选，用于存档）
#[derive(Component, Debug, Default)]
pub struct PlacedObject;

// 新增：标记预览实体
#[derive(Component, Debug, Default)]
pub struct PlacementPreview;

#[derive(Resource)]
struct InteractionConfig {
    placement_offset: f32,
    align_to_normal: bool,
}

impl Default for InteractionConfig {
    fn default() -> Self {
        Self {
            placement_offset: 0.05,
            align_to_normal: true,
        }
    }
}

pub struct ObjectInteractionPlugin;

impl Plugin for ObjectInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin)
            .add_plugins(WireframePlugin::default())
            // 配置全局线框颜色（黄色）
            .insert_resource(WireframeConfig {
                default_color: Color::from(Srgba::rgb(1.0, 1.0, 0.0)),
                ..default()
            })
            .init_resource::<InteractionConfig>()
            .init_resource::<SelectedPlaceable>()
            .add_systems(OnEnter(AppState::InGame), setup) //测试系统
            .add_systems(
                Update,
                (
                    interaction_preview_system,
                    left_click_interaction_system,
                    wireframe_highlight_system,
                    debug_picking,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(
                PreUpdate,
                filter_blocked_picking_hits.before(signal_message_update_system(PointerHits)),
            );
    }
}

///测试系统
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 立方体 1
    let ent1 = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            Name::new("Test Cube 1"),
        ))
        .id();

    // 立方体 2
    let ent2 = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb_u8(20, 50, 255))),
            Transform::from_xyz(2.0, 0.5, 3.0),
            Name::new("Test Cube 2"),
        ))
        .id();
    info!("测试实体1ID::{:?} 实体2ID: {:?}", ent1, ent2);
}
///是去debug系统
fn debug_picking(pointers: Query<&PointerInteraction>) {
    for interaction in &pointers {
        if let Some((entity, hit)) = interaction.get_nearest_hit() {
            info!("拾取到实体: {:?} at {:?}", entity, hit.position);
        }
    }
}

// ────────────────────── 悬停时添加/移除 Wireframe 组件 ──────────────────────
fn wireframe_highlight_system(
    pointers: Query<&PointerInteraction>,
    mut query: Query<(Entity, Option<&Wireframe>), With<Destructible>>,
    mut commands: Commands,
) {
    let mut hovered_entity: Option<Entity> = None;

    // 找到最近的 Destructible 物体
    for interaction in &pointers {
        if let Some((entity, _)) = interaction.get_nearest_hit() {
            if query.get(*entity).is_ok() {
                hovered_entity = Some(*entity);
                break;
            }
        }
    }
    for (entity, wireframe) in &mut query {
        if Some(entity) == hovered_entity {
            // 悬停：添加 Wireframe（显示线框）
            if wireframe.is_none() {
                commands.entity(entity).insert(Wireframe);
            }
        } else {
            // 非悬停：移除 Wireframe（隐藏线框）
            if wireframe.is_some() {
                commands.entity(entity).remove::<Wireframe>();
            }
        }
    }
}

// ────────────────────── 悬停预览系统（红色X + 绿色幽灵） ──────────────────────
fn interaction_preview_system(
    mut gizmos: Gizmos,
    mut commands: Commands,                          // 新增：生成/更新预览实体
    mut meshes: ResMut<Assets<Mesh>>,                // 新增：如果需要自定义 mesh
    mut materials: ResMut<Assets<StandardMaterial>>, // 新增：半透明材质
    mut preview_query: Query<(Entity, &mut Transform, &mut Visibility), With<PlacementPreview>>, // 新增：查询预览实体
    pointers: Query<&PointerInteraction>,
    player_tool: Query<&PlayerTool>,
    selected: Res<SelectedPlaceable>,
    config: Res<InteractionConfig>,
    destructible: Query<(), With<Destructible>>,
) {
    let Ok(tool) = player_tool.single() else {
        return;
    };

    let hit = pointers
        .iter()
        .filter_map(|i| i.get_nearest_hit())
        .min_by_key(|(_, h)| (h.depth * 1000.0) as u32)
        .map(|(entity, hit)| (entity, hit.position, hit.normal));

    let Some((hit_entity, hit_pos, hit_normal)) = hit else {
        // 无击中 → 隐藏预览
        for (_, _, mut visibility) in &mut preview_query {
            *visibility = Visibility::Hidden;
        }
        return;
    };

    // 情况1：手持斧头 + 指向可破坏物 → 红色大 X（Gizmos，不变）
    if *tool == PlayerTool::Axe && destructible.contains(*hit_entity) {
        // 隐藏预览（优先显示 X）
        for (_, _, mut visibility) in &mut preview_query {
            *visibility = Visibility::Hidden;
        }

        let size = 1.0;
        if let Some(hit_pos) = hit_pos {
            gizmos.line(
                hit_pos + Vec3::new(-size, size, 0.0),
                hit_pos + Vec3::new(size, -size, 0.0),
                Color::srgb(1.0, 0.0, 0.0),
            );
            gizmos.line(
                hit_pos + Vec3::new(-size, -size, 0.0),
                hit_pos + Vec3::new(size, size, 0.0),
                Color::srgb(1.0, 0.0, 0.0),
            );
            gizmos.circle(hit_pos, size * 0.8, Color::srgb(1.0, 0.2, 0.2));
        }
        return;
    }

    // 情况2：选中了可放置物 → 生成/更新绿色半透明预览实体
    if selected.is_valid() {
        if let (Some(hit_pos), Some(hit_normal)) = (hit_pos, hit_normal) {
            let place_pos = hit_pos + hit_normal * config.placement_offset;
            let mut transform = Transform::from_translation(place_pos);
            if config.align_to_normal {
                transform.look_to(hit_normal, Vec3::Y);
            }

            // 如果预览实体不存在，生成一个
            let preview_entity = if preview_query.is_empty() {
                let preview_mesh = meshes.add(Plane3d::default().mesh().size(1.0, 1.0));
                let preview_mat = materials.add(StandardMaterial {
                    base_color: Color::srgba(0.3, 1.0, 0.3, 0.5), // 绿色半透明
                    alpha_mode: AlphaMode::Blend,                 // 启用透明混合
                    unlit: true,                                  // 简单预览，不用光照
                    ..default()
                });

                commands
                    .spawn((
                        Mesh3d(preview_mesh),
                        MeshMaterial3d(preview_mat),
                        transform,
                        GlobalTransform::default(),
                        Visibility::Visible,
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        PlacementPreview,
                    ))
                    .id()
            } else {
                let (entity, mut _transfrom, mut _vis) =
                    preview_query.single_mut().expect("获取实体失败");
                entity
            };

            // 更新 Transform
            if let Ok((_, mut preview_transform, mut visibility)) =
                preview_query.get_mut(preview_entity)
            {
                *preview_transform = transform;
                *visibility = Visibility::Visible;
            }
        }
    } else {
        // 无选中 → 隐藏预览
        for (_, _, mut visibility) in &mut preview_query {
            *visibility = Visibility::Hidden;
        }
    }
}

// ────────────────────── 左键交互系统（核心） ──────────────────────
fn left_click_interaction_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
    player_tool: Query<&PlayerTool>,
    selected: Res<SelectedPlaceable>,
    config: Res<InteractionConfig>,
    destructible: Query<(), With<Destructible>>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(tool) = player_tool.single() else {
        return;
    };

    let hit = pointers
        .iter()
        .filter_map(|i| i.get_nearest_hit())
        .min_by_key(|(_, h)| (h.depth * 1000.0) as u32)
        .map(|(entity, hit)| (entity, hit.position, hit.normal));

    let Some((hit_entity, hit_pos, hit_normal)) = hit else {
        return;
    };

    // 优先级1：手持斧头 + 点击可破坏物 → 砍掉
    if *tool == PlayerTool::Axe && destructible.contains(*hit_entity) {
        commands.entity(*hit_entity).despawn();
        info!("Axe: 砍倒物体");
        return;
    }

    // 优先级2：选中可放置物 → 放置
    if selected.is_valid() {
        if let (Some(hit_pos), Some(hit_normal)) = (hit_pos, hit_normal) {
            let place_pos = hit_pos + hit_normal * config.placement_offset;
            let mut transform = Transform::from_translation(place_pos);
            if config.align_to_normal {
                transform.look_to(hit_normal, Vec3::Y);
            }

            commands.spawn((
                SceneRoot(selected.scene.clone()),
                transform,
                Destructible,
                PlacedObject,
                Name::new(format!("Placed {}", selected.name)),
            ));

            info!("Left Click: 放置 {}", selected.name);
            // 可选：消耗一个物品（发事件给背包系统）
            // commands.trigger(ConsumeItem { item_id: selected.id });
        }
    }
}

/// 自定义事件：请求阻挡 UI 的 Picking 传播
#[derive(Event, Clone, Copy, Debug, Message)]
pub struct BlockPickingPropagation {
    pub pointer: PointerId,
}

/// 标记：此 UI 实体需要阻挡 3D Picking 射线
#[derive(Component, Default)]
pub struct UiPickingBlocker;

/// 核心系统：在 PreUpdate 阶段拦截并阻断 UI 的 Picking 事件
fn block_ui_picking(
    blocker_query: Query<(Entity, &UiPickingBlocker)>,
    mut block_events: MessageWriter<BlockPickingPropagation>,
    pointers: Query<&PointerId>,
) {
    // 为所有指针发送阻挡请求（只要有 UI 阻挡器存在）
    if !blocker_query.is_empty() {
        for &pointer in &pointers {
            block_events.write(BlockPickingPropagation { pointer });
        }
    }
}

// 真正拦截 Picking 事件的系统（必须在 PointerHits 事件更新前运行）
fn filter_blocked_picking_hits(
    mut pointer_hits: MessageReader<PointerHits>,
    mut block_events: MessageReader<BlockPickingPropagation>,
    blocker_query: Query<Entity, With<UiPickingBlocker>>,
) {
    // 收集所有需要阻挡的指针
    let blocked_pointers: HashSet<_> = block_events.read().map(|e| e.pointer).collect();

    // 过滤掉所有指向 UI 阻挡器的击中
    for hits in pointer_hits.read() {
        if blocked_pointers.contains(&hits.pointer) {
            // 如果击中的是 UI 阻挡器，清除所有后续 3D 击中
            for (entity, _) in hits.picks.clone() {
                if blocker_query.contains(entity) {
                    // 直接消耗事件，不传播
                    continue;
                }
            }
        }

        // 正常传播非 UI 的击中
        // （这里不需要手动转发，Bevy 会自动处理剩余事件）
    }
}
