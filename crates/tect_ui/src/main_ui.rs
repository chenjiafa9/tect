///主菜单界面
use bevy::prelude::*;
use tect_state::app_state::*;

pub struct MainUiPlugin;

impl Plugin for MainUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

const _SKYBLUE: Color = Color::srgb(0., 0.75, 1.);
const LIGHTSKYBLUE: Color = Color::srgb(0.53, 0.66, 0.71);

//主菜单按钮标记
#[derive(Debug, Component)]
struct MenuButton;

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


fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let font_handle = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands.spawn(Camera2d);

    //游戏菜单按钮
    let options = ["NEW GEANE", "CONTINUE GAME", "ONLINE GAME", "SETING", "ABOUT"];

    // 游戏背景图片
    let bk = commands
        .spawn((
            Node { ..default() },
            Sprite {
                image: asset_server.load("ui_image/BG2.png"),
                ..default()
            },
        ))
        .id();

    // 游戏主菜单选项
    let main_menu = commands
        .spawn(Node {
            width: Val::Percent(30.),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Column,
            display: Display::Flex,
            margin: UiRect::top(Val::Px(30.)),
            ..default()
        })
        .with_children(|parent| {
            for opt in options {
                parent
                    .spawn((
                        Button,
                        Node {
                            width: Val::Percent(70.),
                            height: Val::Percent(8.),
                            border: UiRect::all(Val::Px(5.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(Val::Px(20.)),
                            ..default()
                        },
                        BorderColor::all(Color::BLACK),
                        BorderRadius::all(Val::Px(30.)),
                        BackgroundColor(NORMAL_BUTTON),
                    ))
                    .with_child((
                        Text::new(opt.to_string()),
                        TextFont {
                            font_size: 30.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        MenuButton,
                    ));
            }
        })
        .id();
    commands.insert_resource(MenuData {
        root_entity: main_menu,
    });
    commands.insert_resource(MenuBkCm { bk_entity: bk });
}
