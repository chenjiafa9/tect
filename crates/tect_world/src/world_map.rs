use bevy::color::palettes::css::*;
use bevy::prelude::*;
use tect_assetload::asset_load::*;
use tect_camera::god_view_camera::{calculate_rotation, GodViewCamera, GodViewCameraPlugin};
use tect_control::moving::{Ground, MoveControlPlugin, PlayerMove};
use tect_state::app_state::*;

pub struct WorldScenePlugin;

impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup)
            .add_plugins((GodViewCameraPlugin, MoveControlPlugin));
    }
}
//父
// 初始化测试系统
fn setup(
    mut commands: Commands,
    cameras: Query<(Entity, &Camera), With<Camera>>,
    assets: Res<GameAssets>,
) {
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
    ));
    // 角色
    commands.spawn((
        SceneRoot(assets.player_scene.clone()),
        Transform {
            translation: Vec3::new(5.0, 1.0, 2.0),
            ..default()
        },
        PlayerMove {
            move_speed: 2.0,
            target_position: None,
        },
    ));
    // 场景
    commands.spawn((
        SceneRoot(assets.map.clone()),
        Transform::from_scale(Vec3::splat(1.0)),
        Ground,
    ));
}
