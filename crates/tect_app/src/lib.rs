use bevy::prelude::*;
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
use tect_assetload::asset_load::SmartLoadingPlugin;
use tect_ai::TectAiPlugin;
use tect_state::app_state::*;
use tect_ui::main_ui::*;
use tect_world::world_map::WorldScenePlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tect".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin::default()) //调试界面插件
        .add_plugins(WorldInspectorPlugin::new()) //调试界面插件
        .add_plugins(GameStatePlugin)
        .add_plugins(SmartLoadingPlugin)
        .add_plugins(MainUiPlugin)
        .add_plugins(WorldScenePlugin)
        .add_plugins(TectAiPlugin)
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .run();
}
