///外部使用改移动插件时在需要移动的组件生成时加上PlayerMove，地面组件加上Ground 并应用插件MoveControlPlugin
use bevy::prelude::*;
use tect_state::app_state::*;

pub struct MoveControlPlugin;

impl Plugin for MoveControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (mouse_button_system, character_movement_system).chain());
    }
}

// 组件定义
#[derive(Component)]
pub struct PlayerMove {
    pub move_speed: f32,
    pub target_position: Option<Vec3>,
}



// 资源：用于存储鼠标状态（现在部分状态由 RightMouseAction 管理）
#[derive(Resource)]
struct MouseState {
    // is_right_clicked 和 right_click_position 不再用于判定，仅用于记录点击信息
    is_right_clicked: bool,
    target_is_reach: bool,
    right_click_position: Vec2,
    //鼠标样式动画
    //TODO
}

#[derive(Component)]
pub struct Ground;

// 初始化测试系统，插件实际应用时不挂载该系统
fn setup(mut commands: Commands) {
    // 初始化鼠标状态
    commands.insert_resource(MouseState {
        is_right_clicked: false,
        target_is_reach: false,
        right_click_position: Vec2::ZERO,
    });
}

// 鼠标按键处理系统
fn mouse_button_system(
    mut mouse_state: ResMut<MouseState>,
    mut right_mouse_action: ResMut<RightMouseAction>, // 共享状态
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut gizmos: Gizmos,
    mut player_query: Query<(&mut Transform, &mut PlayerMove)>,
) {
    // 仅当 RightMouseAction 判定为 CharacterMove 时才执行移动逻辑
    if *right_mouse_action != RightMouseAction::CharacterMove {
        // 在这里，我们可以处理 CharacterMove 之后的重置，
        // 或者简单地确保 CharacterMove 逻辑只运行一次。
        return;
    }

    // 重置状态：一旦进入 CharacterMove 逻辑，无论是否找到目标，都意味着点击动作已处理
    // 下一帧开始时，CameraControl 系统会再次设置 AwaitingDecision (如果右键仍按着)，或 None
    *right_mouse_action = RightMouseAction::None;
    
    // 以下是原有的移动逻辑，现在只在判定为 CharacterMove 时执行
    let (camera, camera_transform) = *camera_query;

    if let Some(cursor_position) = window.cursor_position()
        && let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position)
        && let Some(distance) =
            ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    {
        let point = ray.get_point(distance);

        // gizmos绘制为实时绘制，当前在捕捉鼠标按下时只会渲染一帧，基本不可见，后续替换为在鼠标点击点播放一个动画
        gizmos.circle(
            Isometry3d::new(
                point + ground.up() * 0.01,
                Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
            ),
            0.2,
            Color::WHITE,
        );

        mouse_state.is_right_clicked = true;
        mouse_state.right_click_position = cursor_position;

        //保存鼠标点击的目标地点
        for (mut _transform, mut player) in player_query.iter_mut() {
            let target_point = ray.origin + ray.direction * distance;
            player.target_position = Some(target_point);
            mouse_state.target_is_reach = false;
        }
    }
    
    // 释放逻辑：不再需要在这里处理 just_released，因为 CameraControl 已经通过 AwaitingDecision 状态处理了释放的判定。
    // if mouse_button_input.just_released(MouseButton::Right) {
    //     mouse_state.is_right_clicked = false;
    // }
}


// 角色移动系统
fn character_movement_system(
    mut player_query: Query<(&mut Transform, &mut PlayerMove)>,
    mut mouse_state: ResMut<MouseState>,
    time: Res<Time>,
) {
    if mouse_state.target_is_reach {
        return;
    };
    //角色移动逻辑
    for (mut transform, mut player) in player_query.iter_mut() {
        // 如果已经设置了目标位置，则平滑移动过去
        if let Some(target) = player.target_position {
            let direction = target - transform.translation;
            let distance = direction.length();

            if distance > 0.1 {
                let movement = direction.normalize() * player.move_speed * time.delta_secs();
                // 让角色面向移动方向
                let look_direction = Vec3::new(movement.x, 0.0, movement.z).normalize();
                let translation = transform.translation;
                transform.look_at(translation + look_direction, Vec3::Y);
                // 只在XZ平面移动，保持Y坐标不变
                transform.translation.x += movement.x;
                transform.translation.z += movement.z;
            } else {
                mouse_state.target_is_reach = true;
                player.target_position = None;
            }
        }
    }
}
