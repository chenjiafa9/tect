//! 完全独立于 RightMouseAction 的左键交互系统
//! 左键 = 砍树 / 放置物品（由工具 + 快捷栏决定）
//! 右键只负责相机和移动，不参与任何交互

use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::picking::pointer::PointerInteraction;
use bevy::prelude::*;
use tect_assetload::asset_load::GameAssets;

/// 当前手持工具
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerTool {
    #[default]
    None,
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
            .init_resource::<InteractionConfig>()
            .add_systems(
                Update,
                (interaction_preview_system, left_click_interaction_system),
            );
    }
}

// ────────────────────── 悬停预览系统（红色X + 绿色幽灵） ──────────────────────
fn interaction_preview_system(
    mut gizmos: Gizmos,
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
        return;
    };

    // 情况1：手持斧头 + 指向可破坏物 → 红色大 X
    if *tool == PlayerTool::Axe && destructible.contains(*hit_entity) {
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

    // 情况2：选中了可放置物 → 绿色半透明幽灵预览
    if selected.is_valid() {
        if let (Some(hit_pos), Some(hit_normal)) = (hit_pos, hit_normal) {
            let place_pos = hit_pos + hit_normal * config.placement_offset;
            let mut transform = Transform::from_translation(place_pos);
            if config.align_to_normal {
                transform.look_to(hit_normal, Vec3::Y);
            }

            gizmos.scene(
                &SceneRoot {
                    scene: selected.scene.clone(),
                    transform,
                    ..default()
                },
                Color::srgba(0.3, 1.0, 0.3, 0.5),
            );
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
