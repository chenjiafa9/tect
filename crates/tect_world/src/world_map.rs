use bevy::color::palettes::css::*;
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use tect_assetload::asset_load::*;
use tect_camera::god_view_camera::{calculate_rotation, GodViewCamera, GodViewCameraPlugin};
use tect_control::moving::{Ground, MoveControlPlugin, PlayerMove};
use tect_control::object_interaction::{ObjectInteractionPlugin,PlayerTool};
use tect_state::{app_state::*, player::PlayerStats};
use tect_ai::pathfinding::{PathfindingGrid, GridNode};
use tect_ai::npc::{Npc, NpcPatrol};

pub struct WorldScenePlugin;

impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_plugins((
                GodViewCameraPlugin,
                MoveControlPlugin,
                ObjectInteractionPlugin,
            ))
            .add_observer(on_player_scene_loaded);
    }
}
//父
// 初始化测试系统
fn setup(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera), With<Camera>>,
    assets: Res<GameAssets>,
    mut pathfinding_grid: ResMut<PathfindingGrid>,
) {
    // 初始化寻路网格
    pathfinding_grid.grid_size = 1.0;
    pathfinding_grid.world_bounds = (Vec2::new(-50.0, -50.0), Vec2::new(50.0, 50.0));
    
    // 可以在这里添加障碍物
    // pathfinding_grid.add_obstacle(GridNode::new(5, 5));
    
    info!("寻路网格已初始化: 网格大小 = {}, 世界边界 = {:?}", 
          pathfinding_grid.grid_size, pathfinding_grid.world_bounds);
    //点光源
    // commands.spawn((
    //     PointLight {
    //         intensity: 1000_000.0,
    //         color: WHITE.into(),
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(10.0, 200.0, 0.0),
    //     children![(
    //         Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
    //         MeshMaterial3d(materials.add(StandardMaterial {
    //             base_color: WHITE.into(),
    //             emissive: LinearRgba::new(4.0, 0.0, 0.0, 0.0),
    //             ..default()
    //         })),
    //     )],
    // ));

    //  清除所有非游戏主相机的相机（包括默认的）
    for (entity, _camera) in cameras.iter() {
        commands.entity(entity).despawn();
        info!("已清除多余相机: {:?}", entity);
    }

    let camera_data = GodViewCamera::default();

    // 初始化时，根据默认 Yaw 和 Pitch 计算 Transform
    let rotation = calculate_rotation(0.0, camera_data.default_pitch);
    let translation = camera_data.focus + rotation * Vec3::new(0.0, 0.0, camera_data.distance);
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform {
            translation,
            rotation,
            ..default()
        },
        //环境光
        AmbientLight {
            color: WHITE.into(),
            brightness: 1000.0,
            ..default()
        },
        camera_data,
        MeshPickingCamera,//相机拾取标记，只拾取带有Pickable标记的实体
    ));

    // 场景
    commands.spawn((
        SceneRoot(assets.map.clone()),
        Transform::from_scale(Vec3::splat(1.0)),
        Ground,
        // Pickable::IGNORE //避免被选中
    ));
    // 角色
    spawn_player(commands, assets);
    
    // 生成 NPC
    spawn_npcs(commands, assets);
}

// 生成玩家
fn spawn_player(mut commands: Commands, game_assets: Res<GameAssets>) {
    let player_root = commands
        .spawn((
            Transform {
                translation: Vec3::new(5.0, 0.0, 2.0),
                ..default()
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            PlayerMove {
                move_speed: 4.0,
                target_position: None,
            },
            PlayerStats::default(),
            Name::new("PlayerRoot"),
            PlayerTool::default(),
        ))
        .id();

    // 用 with_children + SceneBundle + 监听 SceneInstanceReady
    commands.entity(player_root).with_children(|parent| {
        parent
            .spawn(SceneRoot(game_assets.player_scene.clone()))
            // 关键：监听场景加载完成事件
            .observe(on_player_scene_loaded);
    });
}

// 当玩家 GLTF 场景完全加载完毕（包括 AnimationPlayer 生成）后触发
fn on_player_scene_loaded(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let scene_entity = trigger.entity;

    // 递归找到这个场景下的 AnimationPlayer
    for child in children.iter_descendants(scene_entity) {
        if let Ok(mut player) = players.get_mut(child) {
            // 插入 AnimationGraphHandle（必须！）
            commands.entity(child).insert(AnimationGraphHandle(
                game_assets.player_animations.graph.clone(),
            ));

            // 立即播放 idle 动画
            player.play(game_assets.player_animations.run).repeat();
        }
    }
}

// 生成 NPC
fn spawn_npcs(mut commands: Commands, game_assets: Res<GameAssets>) {
    // NPC 1: 巡逻型 NPC
    let patrol_points = vec![
        Vec3::new(10.0, 0.0, 10.0),
        Vec3::new(20.0, 0.0, 10.0),
        Vec3::new(20.0, 0.0, 20.0),
        Vec3::new(10.0, 0.0, 20.0),
    ];

    let npc1_root = commands
        .spawn((
            Transform {
                translation: Vec3::new(10.0, 0.0, 10.0),
                ..default()
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            Npc::new("巡逻兵", 3.0),
            NpcPatrol::new(patrol_points, 2.0),
            Name::new("NPC_Patrol"),
        ))
        .id();

    // 为 NPC 添加模型（使用玩家模型作为占位）
    commands.entity(npc1_root).with_children(|parent| {
        parent
            .spawn(SceneRoot(game_assets.player_scene.clone()))
            .observe(on_npc_scene_loaded);
    });

    // NPC 2: 静态 NPC（可以后续添加跟随等行为）
    let npc2_root = commands
        .spawn((
            Transform {
                translation: Vec3::new(-5.0, 0.0, 5.0),
                ..default()
            },
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            Npc::new("守卫", 2.5),
            Name::new("NPC_Guard"),
        ))
        .id();

    commands.entity(npc2_root).with_children(|parent| {
        parent
            .spawn(SceneRoot(game_assets.player_scene.clone()))
            .observe(on_npc_scene_loaded);
    });

    info!("已生成 2 个 NPC");
}

// 当 NPC GLTF 场景完全加载完毕后触发
fn on_npc_scene_loaded(
    trigger: On<SceneInstanceReady>,
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
) {
    let scene_entity = trigger.entity;

    // 递归找到这个场景下的 AnimationPlayer
    for child in children.iter_descendants(scene_entity) {
        if let Ok(mut player) = players.get_mut(child) {
            // 插入 AnimationGraphHandle
            commands.entity(child).insert(AnimationGraphHandle(
                game_assets.player_animations.graph.clone(),
            ));

            // NPC 默认播放 idle 或 run 动画
            player.play(game_assets.player_animations.run).repeat();
        }
    }
}
