use bevy::color::palettes::css::*;
use bevy::prelude::*;
use std::f32::consts::PI;
use tect_camera::god_view_camera::{calculate_rotation, GodViewCamera, GodViewCameraPlugin};
use tect_control::moving::{Ground, MoveControlPlugin, PlayerMove};
use tect_state::app_state::*;

pub struct WorldScenePlugin;

impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MoveControlPlugin, GodViewCameraPlugin))
            .add_systems(OnEnter(AppState::InGame), setup);
    }
}

// 初始化测试系统
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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

    let camera_data = GodViewCamera::default();

    // 初始化时，根据默认 Yaw 和 Pitch 计算 Transform
    let rotation = calculate_rotation(0.0, camera_data.default_pitch);
    let translation = camera_data.focus + rotation * Vec3::new(0.0, 0.0, camera_data.distance);
    // camera
    commands.spawn((
        // Camera3d::default(),
        // Transform::from_xyz(0.0, 6.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("rola/rola_walk.glb"))),
        Transform::from_scale(Vec3::new(1.0,1.0,1.0)),
        PlayerMove {
            move_speed: 2.0,
            target_position: None,
        },
    ));

    // 场景
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("scnens/simple_map.glb"))),
        Transform::from_scale(Vec3::splat(1.0)),
        Ground,
    ));
}
