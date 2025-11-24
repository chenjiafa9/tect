use bevy::color::palettes::css::*;
use bevy::prelude::*;
use std::f32::consts::PI;
use tect_camera::god_view_camera::{GodViewCamera, GodViewCameraPlugin,calculate_rotation};
use tect_control::moving::{Ground, MoveControlPlugin, PlayerMove};

pub struct WorldScenePlugin;

impl Plugin for WorldScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((MoveControlPlugin, GodViewCameraPlugin))
            .add_systems(Startup, setup);
    }
}

// 初始化测试系统
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // 光源
    commands.spawn((
        PointLight {
            intensity: 100_000.0,
            color: RED.into(),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 0.0),
        children![(
            Mesh3d(meshes.add(Sphere::new(0.1).mesh().uv(32, 18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: RED.into(),
                emissive: LinearRgba::new(4.0, 0.0, 0.0, 0.0),
                ..default()
            })),
        )],
    ));

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
        camera_data,
    ));

    // 角色
    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("scnens/robot_01.glb"))),
        Transform::from_scale(Vec3::splat(2.0))
            .with_translation(Vec3::new(-2.0, 0.05, -2.1))
            .with_rotation(Quat::from_rotation_y(PI / 2.0)),
        PlayerMove {
            move_speed: 2.0,
            target_position: None,
        },
    ));

    // 场景
    commands.spawn((
        SceneRoot(
            asset_server.load(GltfAssetLabel::Scene(0).from_asset("scnens/mini_diorama_01.glb")),
        ),
        Transform::from_scale(Vec3::splat(10.0)),
        Ground,
    ));
}

