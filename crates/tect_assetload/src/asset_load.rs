use bevy::{asset::LoadState, prelude::*};
use tect_state::app_state::*;

// 启动就必须加载的（体积小，菜单一定用）
#[derive(Resource, Default)]
pub struct BootAssets {
    pub ui_font: Handle<Font>,
    pub menu_bg: Handle<Image>,
}

// 只有真正开始游戏才加载的（体积大，菜单不用）
#[derive(Resource, Default)]
pub struct GameAssets {
    pub player_scene: Handle<Scene>,
    pub animation_graph: Handle<AnimationGraph>,
    pub run_animation: AnimationNodeIndex,
    pub map: Handle<Scene>,
}

// 加载进度通用追踪器
#[derive(Resource, Default)]
pub struct LoadingTracker {
    pub handles: Vec<UntypedHandle>,
    pub total: usize,
}

// UI 标记组件
#[derive(Component)]
struct LoadingRoot;

// 进度条 标记组件
#[derive(Component)]
struct ProgressBar;

// 进度条文本 标记组件
#[derive(Component)]
struct ProgressText;

// ──────────────────────────────
// 3. 智能加载插件（核心）
// ──────────────────────────────
pub struct SmartLoadingPlugin;

impl Plugin for SmartLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BootAssets::default())
            .insert_resource(GameAssets::default())
            .insert_resource(LoadingTracker::default())
            // 第1阶段：启动加载
            .add_systems(OnEnter(AppState::BootLoading), boot_loading_setup)
            .add_systems(
                Update,
                update_progress.run_if(in_state(AppState::BootLoading)),
            )
            .add_systems(
                OnExit(AppState::BootLoading),
                (enter_menu, cleanup_loading_ui),
            )
            // 第2阶段：游戏资源加载（从 Menu 点击「开始游戏」触发）
            .add_systems(OnEnter(AppState::GameLoading), game_loading_setup)
            .add_systems(
                Update,
                update_progress.run_if(in_state(AppState::GameLoading)),
            )
            .add_systems(
                OnExit(AppState::GameLoading),
                (enter_ingame, cleanup_loading_ui),
            );
    }
}

// ──────────────────────────────
//第1阶段：启动加载（只加载菜单资源）
// ──────────────────────────────
fn boot_loading_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut boot_assets: ResMut<BootAssets>,
    mut tracker: ResMut<LoadingTracker>,
) {
    spawn_loading_ui(
        &mut commands,
        "游戏启动中...",
        Color::srgba(0.1, 0.1, 0.2, 0.95),
    );

    let handles = vec![asset_server.load_untyped("ui/BG2.png").untyped()];

    tracker.handles = handles.clone();
    tracker.total = handles.len();

    // 赋值强类型句柄
    boot_assets.ui_font = asset_server.load("fonts/AlibabaPuHuiTi-3-55-Regular.ttf");
    boot_assets.menu_bg = asset_server.load("ui/BG2.png");
}

// ──────────────────────────────
//第2阶段：游戏资源加载（点开始游戏后）
// ──────────────────────────────
fn game_loading_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut game_assets: ResMut<GameAssets>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut tracker: ResMut<LoadingTracker>,
) {
    spawn_loading_ui(
        &mut commands,
        "进入游戏世界...",
        Color::srgba(0.0, 0.1, 0.0, 0.95),
    );

    // 加载你的角色模型和动画
    let player_handle: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("rola/rola_run_2-22.glb"));

    // 构建动画图（只包含跑步动画）
    let mut graph = AnimationGraph::new();
    let clip = asset_server.load(GltfAssetLabel::Animation(0).from_asset("rola/rola_run_2-22.glb"));
    let run_node = graph.add_clip(clip, 1.0, graph.root);
    let graph_handle = graphs.add(graph);

    // 其他大资源
    let handles = vec![
        player_handle.clone().untyped(),
        asset_server.load_untyped("rola/rola_die.glb").untyped(),
        // 加你后续的怪物、特效、音乐等
    ];
    tracker.total = handles.len();
    tracker.handles = handles;

    // 赋值给 GameAssets
    game_assets.player_scene = player_handle;
    game_assets.animation_graph = graph_handle.clone();
    game_assets.run_animation = run_node;
    game_assets.map = asset_server.load(GltfAssetLabel::Scene(0).from_asset("scnens/simple_map.glb"));
}

// ──────────────────────────────
// 6. 通用进度更新系统
// ──────────────────────────────
fn update_progress(
    asset_server: Res<AssetServer>,
    tracker: Res<LoadingTracker>,
    mut bar: Query<&mut Node, With<ProgressBar>>,
    mut text: Query<&mut Text, With<ProgressText>>,
    mut next_state: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    if tracker.total == 0 {
        return;
    }

    let loaded = tracker
        .handles
        .iter()
        .filter(|h| asset_server.is_loaded_with_dependencies(*h))
        .count();

    let percent = loaded as f32 / tracker.total as f32;

    if let Ok(mut node) = bar.single_mut() {
        node.width = Val::Percent(percent * 100.0);
    }
    if let Ok(mut text) = text.single_mut() {
        text.0 = format!("{:.0}%", percent * 100.0);
    }

    if percent >= 1.0 {
        // 自动进入下一状态
        next_state.set(match state.get() {
            AppState::BootLoading => AppState::Menu,
            AppState::GameLoading => AppState::InGame,
            _ => AppState::Menu,
        });
    }
}

// ──────────────────────────────
// 7. 状态跳转
// ──────────────────────────────
fn enter_menu(mut next_state: ResMut<NextState<AppState>>, mut tracker: ResMut<LoadingTracker>) {
    tracker.handles.clear();
    next_state.set(AppState::Menu);
}

fn enter_ingame(
    mut next_state: ResMut<NextState<AppState>>,
    mut tracker: ResMut<LoadingTracker>,
    state: Res<State<AppState>>,
) {
    tracker.handles.clear();
    if *state.get() != AppState::InGame {
        next_state.set(AppState::InGame);
    }
}

// ──────────────────────────────
// 8. UI 生成（复用）
// ──────────────────────────────
fn spawn_loading_ui(commands: &mut Commands, title: &str, bg_color: Color) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                // gap: Val::Px(40.0),
                ..default()
            },
            BackgroundColor(bg_color),
            LoadingRoot,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(title),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // 进度条
            p.spawn((
                Node {
                    width: Val::Px(700.0),
                    height: Val::Px(50.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            ))
            .with_children(|p| {
                p.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.0, 0.8, 0.2)),
                    ProgressBar,
                ));
            });

            p.spawn((
                Text::new("0%"),
                TextFont {
                    font_size: 40.0,

                    ..default()
                },
                TextColor(Color::WHITE),
                ProgressText,
            ));
        });
}

// 清理 UI
fn cleanup_loading_ui(mut commands: Commands, query: Query<Entity, With<LoadingRoot>>) {
    for e in query.iter() {
        commands.entity(e).despawn();
    }
}
