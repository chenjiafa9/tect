use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

// --- 1. 组件、资源和常量定义 ---

/// 标记主 3D 相机并存储其控制状态
#[derive(Component)]
pub struct GodViewCamera {
    /// 相机环绕或聚焦的中心点 (XZ平面)
    pub focus: Vec3,
    /// 相机到焦点点的距离
    pub distance: f32,
    /// 默认的俯仰角（绕 X 轴），例如 -45 度（俯视）
    pub default_pitch: f32,
    /// 临时旋转模式下的鼠标拖拽灵敏度
    pub sensitivity: f32,
}

impl Default for GodViewCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 25.0,
            // 使用弧度：-45度俯视
            default_pitch: -std::f32::consts::FRAC_PI_4,
            sensitivity: 0.005,
        }
    }
}

/// 存储右键拖动时的临时旋转状态
/// 必须是 Resource 或 Local，这里使用 Local 状态，因为它只在 `camera_right_drag_rotate` 系统中使用。
#[derive(Default, Resource)]
struct CameraRotateState {
    /// 是否正在按住右键拖动
    dragging: bool,
    /// 旋转模式下的 Yaw 角（绕 Y 轴）
    yaw: f32,
    /// 旋转模式下的 Pitch 角（绕 X 轴）
    pitch: f32,
}

const EDGE_PAN_THRESHOLD: f32 = 0.01; // 窗口边缘 5% 触发平移
const PAN_SPEED: f32 = 10.0; // 相机平移速度
const ZOOM_SPEED: f32 = 1.0; // 滚轮缩放速度

// --- 2. 插件定义 ---

pub struct GodViewCameraPlugin;

impl Plugin for GodViewCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraRotateState>() // 注册旋转状态资源
            // .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_zoom,
                    camera_edge_pan,
                    camera_right_drag_rotate,
                    // 必须在输入处理之后运行，以应用最终的 Transform
                    update_camera_transform,
                )
                    .chain(), // 链式执行确保顺序
            );
    }
}

// --- 3. Startup 系统：创建相机 ---

fn spawn_camera(mut commands: Commands) {
    let camera_data = GodViewCamera::default();

    // 初始化时，根据默认 Yaw 和 Pitch 计算 Transform
    let rotation = calculate_rotation(0.0, camera_data.default_pitch);
    let translation = camera_data.focus + rotation * Vec3::new(0.0, 0.0, camera_data.distance);

    commands.spawn((
        Camera3d::default(),
        Transform {
            translation,
            rotation,
            ..default()
        },
        camera_data,
    ));
}

// --- 4. Update 系统：滚轮缩放 ---

fn camera_zoom(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera_query: Query<&mut GodViewCamera>,
) {
    let mut camera = match camera_query.single_mut() {
        Ok(c) => c,
        Err(_) => return,
    };

    // `MouseWheel::y` 是滚动的量，正值通常是向上滚动（放大）
    let scroll_y = scroll_events.read().map(|e| e.y).sum::<f32>();

    if scroll_y != 0.0 {
        // 根据距离调整缩放效果，使缩放更平滑自然
        let zoom_factor = camera.distance * 0.05 * ZOOM_SPEED;
        camera.distance -= scroll_y * zoom_factor;

        // 限制缩放范围
        camera.distance = camera.distance.clamp(5.0, 100.0);
    }
}

// --- 5. Update 系统：边缘平移 ---

fn camera_edge_pan(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut GodViewCamera, &Transform)>,
    time: Res<Time>,
    // 检查右键是否被按下，如果按下则不进行边缘平移
    mouse_buttons: Res<ButtonInput<MouseButton>>,
) {
    if mouse_buttons.pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single().expect("Primary window not found");
    let (mut camera, transform) = match camera_query.single_mut() {
        Ok(t) => (t.0, t.1),
        Err(_) => return,
    };

    let mut direction = Vec2::ZERO;

    if let Some(position) = window.cursor_position() {
        let x_percent = position.x / window.width();
        let y_percent = position.y / window.height();

        // X 轴（左右）
        if x_percent < EDGE_PAN_THRESHOLD {
            direction.x -= 1.0;
        } else if x_percent > 1.0 - EDGE_PAN_THRESHOLD {
            direction.x += 1.0;
        }

        // Y 轴（上下，对应世界 Z 轴）
        if y_percent < EDGE_PAN_THRESHOLD {
            direction.y += 1.0; // Y 屏幕坐标减小 (靠近顶部) 对应 Z 世界坐标增大 (向前)
        } else if y_percent > 1.0 - EDGE_PAN_THRESHOLD {
            direction.y -= 1.0; // Y 屏幕坐标增大 (靠近底部) 对应 Z 世界坐标减小 (向后)
        }
    }

    if direction != Vec2::ZERO {
        let move_amount = direction.normalize() * PAN_SPEED * time.delta_secs();

        // 获取相机在 XZ 平面上的“右”向量和“前”向量（通过忽略Y轴旋转）
        let forward_flat = transform.forward().with_y(0.0).normalize();
        let right_flat = transform.right().with_y(0.0).normalize();

        // 在 XZ 平面上移动焦点
        let pan_direction = right_flat * move_amount.x + forward_flat * move_amount.y;
        camera.focus += pan_direction;
    }
}

// --- 6. Update 系统：右键拖动改变视角（环绕） ---

fn camera_right_drag_rotate(
    mut state: ResMut<CameraRotateState>, // 使用 Resource 存储全局状态
    mut camera_query: Query<(&mut GodViewCamera, &Transform)>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut cursors: Single<&mut CursorOptions>,
) {
    let (mut camera, transform) = match camera_query.single_mut() {
        Ok(t) => (t.0, t.1),
        Err(_) => return,
    };

    // let mut window = windows.single_mut().expect("Primary window not found");

    // --- 开始拖动 ---
    if mouse_buttons.just_pressed(MouseButton::Right) {
        state.dragging = true;

        // 在开始拖动时，将当前 Transform 转换为 Yaw/Pitch 初始值
        // 仅适用于 GodViewCamera 的旋转结构 (Yaw * Pitch)
        let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        state.yaw = yaw;
        state.pitch = pitch;

        // 捕获光标以获得无限的鼠标输入
        cursors.grab_mode = CursorGrabMode::Confined;
        cursors.visible = false;
    }

    // --- 更新拖动 ---
    if state.dragging {
        let delta: Vec2 = mouse_motion.read().map(|e| e.delta).sum();

        if delta != Vec2::ZERO {
            // Yaw (绕 Y 轴)
            state.yaw -= delta.x * camera.sensitivity;
            // Pitch (绕 X 轴)，限制在 -89度 到 -1度之间，防止翻转
            state.pitch += delta.y * camera.sensitivity;
            state.pitch = state.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.01, // 接近 -90 度
                -0.01,                               // 接近 0 度 (水平)
            );
        }
    }

    // --- 结束拖动 ---
    if state.dragging && mouse_buttons.just_released(MouseButton::Right) {
        state.dragging = false;

        // ❗ 核心要求：松开后还原到上帝视角默认俯仰角
        state.pitch = camera.default_pitch;

        // 释放光标
        cursors.grab_mode = CursorGrabMode::None;
        cursors.visible = true;
    }
}


// --- 7. Update 系统：应用最终的 Transform ---

/// 计算基于 Yaw 和 Pitch 的旋转 Quat
pub fn calculate_rotation(yaw: f32, pitch: f32) -> Quat {
    // Bevy 的标准旋转顺序通常是 YXZ (Yaw, Pitch, Roll)
    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch)
}

fn update_camera_transform(
    mut camera_query: Query<(&mut Transform, &GodViewCamera)>,
    state: Res<CameraRotateState>,
) {
    let (mut transform, camera) = match camera_query.single_mut() {
        Ok(t) => (t.0, t.1),
        Err(_) => return,
    };

    let rotation: Quat;

    // 如果正在拖动，使用拖动的 Yaw 和 Pitch
    if state.dragging {
        rotation = calculate_rotation(state.yaw, state.pitch);
    }
    // 如果没有拖动，使用拖动后保留的 Yaw 和默认的 Pitch
    else {
        rotation = calculate_rotation(state.yaw, camera.default_pitch);
    }

    // 相机位置 = 焦点 + 旋转后的 (0, 0, 距离) 向量
    // 旋转向量 (0, 0, distance) 意味着它会沿着 Z 轴负方向（向后）移动，从焦点处拉开
    let translation = camera.focus + rotation * Vec3::new(0.0, 0.0, camera.distance);

    transform.translation = translation;
    transform.rotation = rotation;
}
