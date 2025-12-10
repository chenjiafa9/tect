// src/tect_control/object_interaction.rs
//! 完全独立的物体交互插件：
//! - 左键：破坏任意带有 Destructible 的物体（支持复杂模型）
//! - 右键：在任意表面放置指定模型（自动贴合法线）

use bevy::{picking::pointer::PointerInteraction, prelude::*};
use bevy::picking::mesh_picking::MeshPickingPlugin;
use bevy::scene::SceneInstanceReady;
use tect_assetload::asset_load::GameAssets;
use serde::{Deserialize, Serialize};

/// 标记可被放置的物体（用于后续扩展：存档、拾取等）
#[derive(Debug, Default,Component, Serialize, Deserialize, Reflect)]
pub struct PlacedObject;

/// 标记可被左键破坏的物体（树、石头、箱子……任意模型都可以）
#[derive(Debug, Default,Component, Serialize, Deserialize, Reflect)]
pub struct Destructible;

/// 放置系统需要的配置（可通过 Resource 自定义）
#[derive(Resource)]
pub struct PlacementConfig {
    /// 要放置的模型（支持任意 .glb/.gltf）
    pub scene: Handle<Scene>,
    /// 放置时的偏移距离（避免嵌入表面）
    pub offset: f32,
    /// 是否自动对齐法线（让树垂直于斜坡）
    pub align_to_normal: bool,
}

impl Default for PlacementConfig {
    fn default() -> Self {
        Self {
            scene: Handle::default(),
            offset: 0.1,
            align_to_normal: true,
        }
    }
}

/// 完整的物体交互插件（直接加到 App 即可）
pub struct ObjectInteractionPlugin;

impl Plugin for ObjectInteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            // Bevy 0.17 内置的高性能 Mesh 拾取
            .add_plugins(MeshPickingPlugin)
            // 注册组件
            .register_type::<Destructible>()
            .register_type::<PlacedObject>()
            // 初始化配置资源（GameAssets 加载完后再赋值）
            .init_resource::<PlacementConfig>()
            // 系统
            .add_systems(Update, (
                init_placement_config.run_if(resource_added::<GameAssets>),
                object_interaction_system,
            ))
            // 可选：物体加载完成回调（比如播放音效、加刚体）
            .add_observer(on_placed_object_loaded);
    }
}

/// 第一次拿到 GameAssets 时，把配置填好
fn init_placement_config(
    mut config: ResMut<PlacementConfig>,
    assets: Res<GameAssets>,
) {
    if config.scene.is_strong() {
        return; // 已经初始化过
    }
    // 这里放你想右键放置的模型（树、石头、箱子、灯……随便换）
    config.scene = assets.player_scene.clone(); 
    info!("ObjectInteraction: 放置模型已配置");
}

/// 核心交互系统
fn object_interaction_system(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    pointers: Query<&PointerInteraction>,
    config: Res<PlacementConfig>,
    // 可破坏的物体
    destructible_query: Query<(), With<Destructible>>,
) {
    // 获取最近的拾取结果
    let hit = pointers
        .iter()
        .filter_map(|i| i.get_nearest_hit())
        .min_by_key(|(_, hit)| (hit.depth * 1000.0) as u32)
        .map(|(entity, hit)| (entity, hit.position, hit.normal));

    let Some((hit_entity, hit_pos, hit_normal)) = hit else {
        return;
    };

    // 左键：破坏
    if mouse.just_pressed(MouseButton::Left) {
        if destructible_query.contains(hit_entity) {
            commands.entity(hit_entity).despawn_recursive();
            info!("物体已破坏: {:?}", hit_entity);
            return;
        }
    }

    // 右键：放置
    if mouse.just_pressed(MouseButton::Right) {
        if config.scene.is_valid() {
            let place_pos = hit_pos + hit_normal * config.offset;

            let mut transform = Transform::from_translation(place_pos);
            if config.align_to_normal {
                // 让物体 Y 轴对齐法线（树长在斜坡上也垂直）
                transform.look_to(hit_normal, Vec3::Y);
            }

            commands.spawn((
                SceneBundle {
                    scene: config.scene.clone(),
                    transform,
                    ..default()
                },
                Destructible,
                PlacedObject,
                Name::new("Placed Object"),
            ));

            info!("物体已放置: {:?}", place_pos);
        } else {
            warn!("PlacementConfig.scene 未加载！");
        }
    }
}

/// 可选：物体加载完成后的回调（比如加刚体、播放音效）
fn on_placed_object_loaded(
    trigger: Trigger<SceneInstanceReady>,
    mut commands: Commands,
) {
    let entity = trigger.entity();
    info!("放置物体加载完成: {:?}", entity);

    // 示例：自动添加物理（如果你用了 bevy_xpbd_3d）
    // commands.entity(entity).insert(RigidBody::Dynamic);
}