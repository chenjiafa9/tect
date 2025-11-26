use bevy::prelude::*;
use tect_state::app_state::*;
use tect_ui::main_ui::*;
use tect_world::world_map::WorldScenePlugin;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldScenePlugin)
        .add_plugins(GameStatePlugin)
        .add_plugins(MainUiPlugin)
        .run();
}
