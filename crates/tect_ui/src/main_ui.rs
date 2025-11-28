///主菜单界面
use bevy::prelude::*;
use tect_state::app_state::*;

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Menu), setup_menu)
            .add_systems(Update, menu_button_system.run_if(in_state(AppState::Menu)))
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

// ui_style.rs 或直接放在文件顶部
const BG_COLOR: Color = Color::srgb(0.05, 0.05, 0.12);
const PANEL_COLOR: Color = Color::srgba(0.1, 0.1, 0.2, 0.92);
const NORMAL_BUTTON: Color = Color::srgba(0.15, 0.15, 0.35, 0.8);
const HOVER_BUTTON: Color = Color::srgba(0.25, 0.75, 0.95, 0.9);
const PRESSED_BUTTON: Color = Color::srgba(0.35, 0.85, 1.0, 1.0);
const TEXT_COLOR: Color = Color::srgb(0.9, 0.9, 0.95);
const ACCENT_COLOR: Color = Color::srgb(0.0, 0.8, 1.0);
const _SKYBLUE: Color = Color::srgb(0., 0.75, 1.);

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(Component)]
struct MenuCamera; // 标记菜单专用的 2D 相机

//主菜单按钮标记
#[derive(Component)]
pub enum MenuButtonAction {
    NewGame,
    ContinueGame,
    OnlineGame,
    OpenSettings,
    OpenAbout,
    Quit,
}

//主菜单按钮实体
#[derive(Resource)]
pub struct MenuData {
    pub root_entity: Entity,
}
///主菜单背景
#[derive(Resource)]
pub struct MenuBkCm {
    pub bk_entity: Entity,
}

///主菜单渲染
fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuOptions>>,
) {
    //  生成菜单专用的 2D UI 相机（
    commands.spawn((
        Camera2d::default(),
        Camera {
            // 确保在所有 3D 相机之上
            order: 999,
            clear_color: ClearColorConfig::None, // 透明，让背景图显示
            ..default()
        },
        MenuCamera,
        Name::new("Menu UI Camera"),
    ));

    // 根节点：全屏背景
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            // BackgroundColor(BG_COLOR),
            Sprite {
                image: asset_server.load("ui_image/BG2.png"),
                ..default()
            },
            Name::new("Menu Root"),
        ))
        .with_children(|parent| {
            // 标题
            // parent.spawn((
            //     Text::new("MY AWESOME GAME"),
            //     TextFont {
            //         font: asset_server.load("fonts/AlibabaPuHuiTi-3-55-Regular.ttf"), 
            //         font_size: 80.0,
            //         ..default()
            //     },
            //     TextColor(TEXT_COLOR),
            //     Node {
            //         margin: UiRect::bottom(Val::Px(60.0)),
            //         ..default()
            //     },
            //     // 进场动画用
            // ));

            // 半透明毛玻璃主面板
            parent
                .spawn((
                    Node {
                        width: Val::Px(420.0),
                        height: Val::Px(560.0),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(PANEL_COLOR),
                    BorderRadius::all(Val::Px(24.0)),
                    BorderColor::all(ACCENT_COLOR.with_alpha(0.3)),
                    Outline::new(Val::Px(2.0), Val::Px(8.0), ACCENT_COLOR.with_alpha(0.2)),
                    MainMenuRoot,
                ))
                .with_children(|panel| {
                    let options = [
                        ("NEW GAME", MenuButtonAction::NewGame),
                        ("CONTINUE", MenuButtonAction::ContinueGame),
                        ("ONLINE", MenuButtonAction::OnlineGame),
                        ("SETTINGS", MenuButtonAction::OpenSettings),
                        ("ABOUT", MenuButtonAction::OpenAbout),
                        ("QUIT", MenuButtonAction::Quit),
                    ];

                    for (label, action) in options {
                        panel
                            .spawn((
                                Button,
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Px(68.0),
                                    margin: UiRect::vertical(Val::Px(12.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                BackgroundColor(NORMAL_BUTTON),
                                BorderRadius::all(Val::Px(16.0)),
                                BorderColor::all(ACCENT_COLOR.with_alpha(0.4)),
                                Outline::new(Val::Px(1.0), Val::Px(4.0), Color::NONE),
                                action,
                            ))
                            .with_child((
                                Text::new(label),
                                TextFont {
                                    // font: asset_server.load("fonts/AlibabaPuHuiTi-3-55-Regular"),
                                    font_size: 32.0,
                                    ..default()
                                },
                                TextColor(TEXT_COLOR),
                            ));
                    }
                });
        });

    // 初始进入主菜单
    next_menu_state.set(MenuOptions::NewGame); // 或你想默认高亮的
}

///按钮点击逻辑
fn menu_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &MenuButtonAction,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_menu_state: ResMut<NextState<MenuOptions>>,
    mut exit: MessageWriter<AppExit>,
) {
    for (interaction, action, mut bg, mut border) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg = PRESSED_BUTTON.into();
                border.set_all(ACCENT_COLOR);

                match action {
                    MenuButtonAction::NewGame => {
                        next_app_state.set(AppState::InGame);
                    }
                    MenuButtonAction::ContinueGame => {
                        // 加载存档逻辑
                        next_app_state.set(AppState::InGame);
                    }
                    MenuButtonAction::OnlineGame => {
                        next_menu_state.set(MenuOptions::OnlineGame);
                    }
                    MenuButtonAction::OpenSettings => {
                        next_menu_state.set(MenuOptions::Setting);
                    }
                    MenuButtonAction::OpenAbout => {
                        next_menu_state.set(MenuOptions::About);
                    }
                    MenuButtonAction::Quit => {
                        exit.write(AppExit::Success);
                    }
                }
            }
            Interaction::Hovered => {
                *bg = HOVER_BUTTON.into();
                border.set_all(ACCENT_COLOR);
            }
            Interaction::None => {
                *bg = NORMAL_BUTTON.into();
                border.set_all(ACCENT_COLOR.with_alpha(0.4));
            }
        }
    }
}



// ────────────────────────────── 退出菜单：清除 UI + 相机 ──────────────────────────────
fn cleanup_menu(
    mut commands: Commands,
    roots: Query<Entity, With<MainMenuRoot>>,
    cameras: Query<Entity, With<MenuCamera>>,
) {
    // 删除主菜单面板（会递归删除所有子节点）
    for entity in &roots {
        commands.entity(entity).despawn();
    }

    // 删除菜单专用相机
    for entity in &cameras {
        commands.entity(entity).despawn();
    }

    // 可选：也清除背景图（如果你想更干净）
    // commands.entity(background).despawn();
}