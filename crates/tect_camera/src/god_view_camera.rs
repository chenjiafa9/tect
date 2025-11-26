use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    time::Stopwatch,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use tect_state::app_state::*;

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
    /// 旋转模式下的 Yaw 角（绕 Y 轴）
    yaw: f32,
    /// 旋转模式下的 Pitch 角（绕 X 轴）
    pitch: f32,
    pub last_manual_pitch: f32, // 记住玩家最后手动设置的 pitch
    pub has_ever_dragged: bool, // 标记是否曾经拖动过
}

const EDGE_PAN_THRESHOLD: f32 = 0.005; // 窗口边缘 0.05% 触发平移
const PAN_SPEED: f32 = 5.0; // 相机平移速度
const ZOOM_SPEED: f32 = 1.0; // 滚轮缩放速度
                             // 拖动判定阈值（例如 200 毫秒）
const DRAG_THRESHOLD_TIME: f32 = 0.3;
// 鼠标最小移动距离阈值（防止微小抖动触发）
const DRAG_THRESHOLD_DISTANCE: f32 = 5.0; // 5 像素的移动

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
                    .run_if(in_state(AppState::InGame))
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
        camera.distance = camera.distance.clamp(5.0, 30.0);
    }
}

// --- 5. Update 系统：边缘平移 ---

fn camera_edge_pan(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut camera_query: Query<(&mut GodViewCamera, &Transform)>,
    time: Res<Time>,
    // 检查右键是否被按下，如果按下则不进行边缘平移
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    // 引入 RightMouseAction 资源
    right_mouse_action: Res<RightMouseAction>,
) {
    // 只有在右键未按下且未处于拖动状态时才进行边缘平移
    if mouse_buttons.pressed(MouseButton::Right)
        || *right_mouse_action == RightMouseAction::CameraDrag
    {
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

// // --- 6. Update 系统：右键拖动改变视角（环绕）或判定动作 ---
// /// 该系统负责判定右键是拖动 (CameraDrag) 还是点击 (CharacterMove)，并执行 CameraDrag 动作。
/// 右键行为：短促点击 → 移动角色；按住并拖动 → 旋转相机
fn camera_right_drag_rotate(
    mut state: ResMut<CameraRotateState>,
    mut right_mouse: ResMut<RightMouseAction>,
    mut drag_timer: Local<f32>, // 取代复杂的 Timer
    mut has_moved_significantly: Local<bool>,

    camera_q: Query<(&GodViewCamera, &Transform)>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion_events: MessageReader<MouseMotion>,
    mut cursor: Single<&mut CursorOptions>,
    time: Res<Time>,
) {
    let (camera, transform) = match camera_q.single() {
        Ok(v) => v,
        Err(_) => return,
    };

    // 读取本帧所有鼠标移动
    let motion_delta: Vec2 = motion_events.read().map(|e| e.delta).sum();

    // === 1. 按下瞬间 ===
    if mouse.just_pressed(MouseButton::Right) {
        *right_mouse = RightMouseAction::PressedJustNow;
        *drag_timer = 0.0;
        *has_moved_significantly = false;
        return;
    }

    // === 2. 处理按住过程中的每一帧 ===
    if mouse.pressed(MouseButton::Right) {
        match *right_mouse {
            RightMouseAction::PressedJustNow | RightMouseAction::WaitingForDecision => {
                // 累计时间与移动距离
                *drag_timer += time.delta_secs();
                if motion_delta.length() > DRAG_THRESHOLD_DISTANCE {
                    *has_moved_significantly = true;
                }

                // 一旦满足“明显拖动”条件，立即进入拖动模式
                if *drag_timer > DRAG_THRESHOLD_TIME || *has_moved_significantly {
                    *right_mouse = RightMouseAction::CameraDrag;

                    // 初始化当前角度
                    let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
                    state.yaw = yaw;
                    state.pitch = pitch;

                    // 捕获光标
                    cursor.grab_mode = CursorGrabMode::Confined;
                    cursor.visible = false;
                } else {
                    // 还在犹豫期
                    *right_mouse = RightMouseAction::WaitingForDecision;
                }
            }

            RightMouseAction::CameraDrag => {
                // 正在拖动 → 实时更新角度
                if motion_delta != Vec2::ZERO {
                    state.yaw -= motion_delta.x * camera.sensitivity;
                    state.pitch -= motion_delta.y * camera.sensitivity;
                    state.pitch = state.pitch.clamp(
                        -std::f32::consts::FRAC_PI_2 + 0.01,
                        -0.01, // 或者使用 camera.max_pitch 上限
                    );
                }
            }

            _ => {}
        }
        return; // 按住时不处理松开
    }

    // === 3. 松开瞬间（只有在非拖动模式下才视为点击）===
    if mouse.just_released(MouseButton::Right) {
        match *right_mouse {
            RightMouseAction::PressedJustNow | RightMouseAction::WaitingForDecision => {
                // 短促点击 → 触发角色移动
                *right_mouse = RightMouseAction::CharacterMove;
                // 移动系统会在下一系统看到这个状态并处理，然后把它清掉
            }
            RightMouseAction::CameraDrag => {
                // 结束拖动，保持当前角度（重要！）
                *right_mouse = RightMouseAction::None;

                // 释放光标
                cursor.grab_mode = CursorGrabMode::None;
                cursor.visible = true;
            }
            _ => {
                *right_mouse = RightMouseAction::None;
            }
        }
    }
}

// --- 7. Update 系统：应用最终的 Transform ---

/// 计算基于 Yaw 和 Pitch 的旋转 Quat
fn update_camera_transform(
    mut camera_query: Query<(&mut Transform, &GodViewCamera)>,
    mut state: ResMut<CameraRotateState>, // 注意要 mut！
    right_mouse_action: Res<RightMouseAction>,
) {
    let (mut transform, camera) = match camera_query.single_mut() {
        Ok(v) => v,
        Err(_) => return,
    };

    let (current_yaw, current_pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

    // === 第一帧初始化：从当前相机姿态读取初始值 ===
    if !state.has_ever_dragged {
        state.yaw = current_yaw;
        state.pitch = current_pitch;
        state.last_manual_pitch = current_pitch;
        state.has_ever_dragged = true; // 防止下次再初始化
    }

    let target_yaw: f32;
    let target_pitch: f32;

    if *right_mouse_action == RightMouseAction::CameraDrag {
        // 拖动中：实时使用 state 中的值（已在拖动系统里更新）
        target_yaw = state.yaw;
        target_pitch = state.pitch;

        // 重要！实时记录玩家手动调整的 pitch
        state.last_manual_pitch = state.pitch;
    } else {
        // 非拖动状态：
        // - Yaw：保持玩家最后拖动时的 yaw（自由旋转）
        // - Pitch：保持玩家最后手动调整的 pitch（不再回 default！）
        target_yaw = state.yaw;
        target_pitch = state.last_manual_pitch;
    }

    // 限制 pitch 范围（防止翻转）
    let target_pitch = target_pitch.clamp(
        -std::f32::consts::FRAC_PI_2 + 0.05,
        -0.05, // 或者使用 camera.min_pitch / max_pitch
    );

    let rotation = Quat::from_euler(EulerRot::YXZ, target_yaw, target_pitch, 0.0);

    // 相机围绕焦点旋转
    let offset = rotation * Vec3::new(0.0, 0.0, camera.distance);
    transform.translation = camera.focus + offset;
    transform.rotation = rotation;
}

pub fn calculate_rotation(yaw: f32, pitch: f32) -> Quat {
    // Bevy 的标准旋转顺序通常是 YXZ (Yaw, Pitch, Roll)
    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch)
}
