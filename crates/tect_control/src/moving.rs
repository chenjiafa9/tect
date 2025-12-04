///外部使用改移动插件时在需要移动的组件生成时加上PlayerMove，地面组件加上Ground 并应用插件MoveControlPlugin
use bevy::prelude::*;
use tect_assetload::asset_load::*;
use tect_state::app_state::*;

pub struct MoveControlPlugin;

impl Plugin for MoveControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), load_click_effect_assets)
            .add_systems(
                Update,
                (
                    mouse_button_system,
                    character_movement_system,
                    control_run_animation_system,
                )
                    .run_if(in_state(AppState::InGame))
                    .chain(),
            );
    }
}

// 组件定义
#[derive(Component)]
pub struct PlayerMove {
    pub move_speed: f32,
    pub target_position: Option<Vec3>,
}

// ──────────────────────────────────────────────────────────────
// 1. 资源定义：右键动画
// ──────────────────────────────────────────────────────────────
#[derive(Resource)]
pub struct ClickEffectAssets {
    pub scene: Handle<Scene>,
    pub graph: Handle<AnimationGraph>,
    pub click_animation: AnimationNodeIndex, // 我们只用一个“Click”动画
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

// 新增：标记角色当前是否应该播放跑步动画
#[derive(Component, Default)]
pub struct IsMoving;

// 初始化资源
fn load_click_effect_assets(mut commands: Commands, assets: Res<GameAssets>) {
    commands.insert_resource(ClickEffectAssets {
        scene: assets.player_scene.clone(),
        graph: assets.animation_graph.clone(),
        click_animation: assets.run_animation,
    });
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
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerMove), With<PlayerMove>>,
    mut commands: Commands,
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
        mouse_state.is_right_clicked = true;
        mouse_state.right_click_position = cursor_position;

        //保存鼠标点击的目标地点
        for (entity, mut _transform, mut player) in player_query.iter_mut() {
            let target_point = ray.origin + ray.direction * distance;
            player.target_position = Some(target_point);
            mouse_state.target_is_reach = false;
            commands.entity(entity).insert(IsMoving);
        }
    }
}

// 角色移动系统
fn character_movement_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &mut Transform, &mut PlayerMove, &Children)>,
    mut mouse_state: ResMut<MouseState>,
    time: Res<Time>,
) {
    if mouse_state.target_is_reach {
        return;
    };
    //角色移动逻辑
    for (entity, mut transform, mut player, _children) in player_query.iter_mut() {
        // 如果已经设置了目标位置，则平滑移动过去
        if let Some(target) = player.target_position {
            let direction = target - transform.translation;
            let distance = direction.length();
            let translation = transform.translation;

            if distance > 0.2 {
                let move_vec = direction.normalize() * player.move_speed * time.delta_secs();
                let look_dir = Vec3::new(move_vec.x, 0.0, move_vec.z).normalize_or_zero();

                // 面向移动方向
                if look_dir.length_squared() > 0.0 {
                    transform.look_at(translation - look_dir, Vec3::Y);
                }

                // 只移动 XZ
                transform.translation += move_vec.with_y(0.0);

                mouse_state.target_is_reach = false;
            } else {
                // 到达目标
                player.target_position = None;
                mouse_state.target_is_reach = true;

                // 移除 IsMoving → 动画系统会暂停动画
                commands.entity(entity).remove::<IsMoving>();
            }
        }
    }
}

// ========================
// 关键：根据 IsMoving 组件控制动画播放/暂停
// ========================
fn control_run_animation_system(
    effect_assets: Res<ClickEffectAssets>,
    moving_query: Query<(), With<IsMoving>>, // 玩家根有 IsMoving
    mut player_query: Query<&mut AnimationPlayer>,
) {
    let should_play = !moving_query.is_empty();
    for mut player in player_query.iter_mut() {
        if should_play {
            // 正在移动 → 播放跑步动画（只 play 一次，避免重复触发）
            if player.animation(effect_assets.click_animation).is_none() {
                player.play(effect_assets.click_animation).repeat();
            }
        } else {
            // 停止移动 → 暂停或清空动画（推荐暂停，更自然）
            if player.animation(effect_assets.click_animation).is_some() {
                player.stop(effect_assets.click_animation);
            }
        }
    }
}
