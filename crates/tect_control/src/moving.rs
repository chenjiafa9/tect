use bevy::animation::{AnimationEvent, AnimationTargetId, RepeatAnimation};
use bevy::asset::AssetContainer;
use bevy::color::palettes::css::WHITE;
use bevy::gltf::Gltf;
///外部使用改移动插件时在需要移动的组件生成时加上PlayerMove，地面组件加上Ground 并应用插件MoveControlPlugin
use bevy::prelude::*;
///描述：当前动画的加载与保存以及动画播放存在问题，与bevy0.17官方示例存在区别，且无法清除播放完的动画，动画事件未成功添加
use std::time::Duration;
use tect_state::app_state::*;

pub struct MoveControlPlugin;

impl Plugin for MoveControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup, load_click_effect_assets))
            .add_systems(
                Update,
                (
                    mouse_button_system,
                    character_movement_system,
                    setup_click_effect_once_loaded,
                    // setup_scene_once_loaded,
                    despawn_finished_click_effects,
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

// 初始化资源
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
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    window: Single<&Window>,
    mut player_query: Query<(&mut Transform, &mut PlayerMove)>,
    click_effect_assets: Res<ClickEffectAssets>,
    mut commands: Commands,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
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
        for (mut _transform, mut player) in player_query.iter_mut() {
            let target_point = ray.origin + ray.direction * distance;
            player.target_position = Some(target_point);
            mouse_state.target_is_reach = false;
        }
        start_stop_onclick(click_effect_assets, animation_players);

        // —— 新增：生成外部动画特效 ——
        // spawn_click_effect(
        //     &mut commands,
        //     &click_effect_assets,
        //     point,
        //     ground.up().as_vec3(),
        // );
    }
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

///初始化右键动画资源
pub fn load_click_effect_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let scene_handle: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("rola/rola_run_2-22.glb"));

    let (graph, animation_indices) = AnimationGraph::from_clips([
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("rola/rola_run_2-22.glb"))
    ]);
    let graph_handle = graphs.add(graph);
    commands.insert_resource(ClickEffectAssets {
        scene: scene_handle,
        graph: graph_handle,
        click_animation: animation_indices[0],
    });
}

///角色跑步动画启动
pub fn start_stop_onclick(
    effect_assets: Res<ClickEffectAssets>,
    mut animation_players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (mut player, mut _transitions) in &mut animation_players {
        let playing_animation = player.animation_mut(effect_assets.click_animation).unwrap();
        if playing_animation.is_paused() {
            playing_animation.resume();
        } else {
            playing_animation.pause();
        }
    }
}

// ──────────────────────────────────────────────────────────────
// 生成特效函数（在鼠标系统里调用）
// ──────────────────────────────────────────────────────────────
fn spawn_click_effect(
    commands: &mut Commands,
    effect_assets: &ClickEffectAssets,
    position: Vec3,
    ground_normal: Vec3,
) {
    commands.spawn((
        SceneRoot(effect_assets.scene.clone()),
        Transform::from_translation(position + ground_normal * 0.02)
            .looking_to(ground_normal, Vec3::Y),
        GlobalTransform::default(),
        Visibility::Visible,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        // 这些组件会在 setup_effect_once_loaded 中被填充
        // 所以这里先占位，实际会在加载完成后插入
    ));
}


// ──────────────────────────────────────────────────────────────
// 关键系统：场景加载完成后绑定动画图 + 播放一次 + 自动销毁
// ──────────────────────────────────────────────────────────────
fn setup_click_effect_once_loaded(
    mut commands: Commands,
    effect_assets: Res<ClickEffectAssets>,
    animations: Res<ClickEffectAssets>,
    graphs: Res<Assets<AnimationGraph>>,
    mut clips: ResMut<Assets<AnimationClip>>,
    // 只查询本次新生成的特效实体中，刚刚添加了 AnimationPlayer 的
    mut query: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in query.iter_mut() {
        // 安全检查：确保是我们生成的特效
        if commands.get_entity(entity).is_err() {
            continue;
        }
        let graph = graphs.get(&animations.graph).unwrap();
        let running_animation = get_clip(animations.click_animation, graph, &mut clips);

        let mut entity_cmds = commands.entity(entity);

        // 插入动画图
        entity_cmds.insert(AnimationGraphHandle(effect_assets.graph.clone()));

        // 创建过渡控制器
        let mut transitions = AnimationTransitions::new();

        // 立即播放“Click”动画，0.2秒淡入，播放一次
        transitions
            .play(
                player.as_mut(),
                effect_assets.click_animation,
                Duration::from_millis(0),
            )
            .set_repeat(RepeatAnimation::Count(1));

        entity_cmds.insert(transitions);
        
    }
}

fn get_clip<'a>(
    node: AnimationNodeIndex,
    graph: &AnimationGraph,
    clips: &'a mut Assets<AnimationClip>,
) -> &'a mut AnimationClip {
    let node = graph.get(node).unwrap();
    let clip = match &node.node_type {
        AnimationNodeType::Clip(handle) => clips.get_mut(handle),
        _ => unreachable!(),
    };
    clip.unwrap()
}
// ──────────────────────────────────────────────────────────────
// 清理系统：监听动画结束事件并删除实体（官方推荐方式）
// ──────────────────────────────────────────────────────────────
fn despawn_finished_click_effects(
    mut commands: Commands,
    // mut click: On<OnClick>,
) {
    // commands.entity().despawn();
}
