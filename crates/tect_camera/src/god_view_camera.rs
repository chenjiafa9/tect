use bevy::{
    input::mouse::{MouseMotion, MouseWheel}, prelude::*, time::Stopwatch, window::{CursorGrabMode, CursorOptions, PrimaryWindow}
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
    /// 记录右键按下的计时器
    press_timer: Stopwatch, 
    /// 标记右键是否处于“刚刚按下，等待判定”的状态
    awaiting_drag_start: bool,
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
///该系统和右键点击控制角色移动有冲出，当鼠标右键按下时角色已经移动，应存在一个状态控制视角拖动与角色移动进行互斥，通过判定之后
/// 指挥运行其中一个系统
fn camera_right_drag_rotate(
    mut state: ResMut<CameraRotateState>, 
    mut camera_query: Query<(&mut GodViewCamera, &Transform)>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut cursors: Single<&mut CursorOptions>,
    time: Res<Time>, // 引入 Time 资源用于更新计时器
) {

    let (camera, transform) = match camera_query.single_mut() {
        Ok(t) => (t.0, t.1),
        Err(_) => return,
    };
       
    // 累积鼠标移动的 delta
    let delta: Vec2 = mouse_motion.read().map(|e| e.delta).sum();
    
    // --- 逻辑 A: 处理右键按下/等待 ---

    if mouse_buttons.just_pressed(MouseButton::Right) {
        // 1. 启动等待状态
        state.awaiting_drag_start = true;
        state.press_timer.reset(); // 重置计时器
        state.press_timer.unpause();
    }
    
    if state.awaiting_drag_start {
        // 2. 更新计时器
        state.press_timer.tick(time.delta());
        
        // 3. 检查是否达到时间或移动阈值
        let moved_enough = delta.length() > DRAG_THRESHOLD_DISTANCE;
        let timed_out = state.press_timer.elapsed_secs() >= DRAG_THRESHOLD_TIME;
        
        if moved_enough || timed_out {
            // 如果时间到或鼠标已移动，则判定为拖动意图，进入拖动模式
            
            // 停止等待
            state.awaiting_drag_start = false;
            state.press_timer.pause();
            
            // 切换到拖动状态 (与旧版的开始拖动逻辑相同)
            state.dragging = true;
            let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            state.yaw = yaw;
            state.pitch = pitch;
            
            // 捕获光标 (延迟后才隐藏光标)
            cursors.grab_mode = CursorGrabMode::Confined;
            cursors.visible = false;
        }
    }
    
    // --- 逻辑 B: 处理右键松开 ---

    if mouse_buttons.just_released(MouseButton::Right) {
        state.press_timer.pause();
        
        if state.awaiting_drag_start {
            // 4. 如果在计时结束前松开，则认为是“点击/平移”操作
            state.awaiting_drag_start = false;
            
            // ⚠️ 注意: 在这里，您可以选择触发一个“平移命令”或“单位命令”。
            // 由于相机平移由 `camera_edge_pan` 或其他系统处理，这里不做额外动作。
            // 确保不捕获光标，光标状态不变。
            return;
        }

        if state.dragging {
            // 5. 如果处于拖动模式下松开，则结束拖动
            state.dragging = false;
            
            // 还原到上帝视角默认俯仰角
            state.pitch = camera.default_pitch; 
            
            // 释放光标
            cursors.grab_mode = CursorGrabMode::None;
            cursors.visible = true;
        }
    }
    
    // --- 逻辑 C: 更新拖动中状态 ---

    if state.dragging {
        // 累积拖动中产生的 delta
        if delta != Vec2::ZERO {
            state.yaw -= delta.x * camera.sensitivity;
            state.pitch += delta.y * camera.sensitivity;
            state.pitch = state.pitch.clamp(
                -std::f32::consts::FRAC_PI_2 + 0.01,
                -0.01,
            );
        }
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
