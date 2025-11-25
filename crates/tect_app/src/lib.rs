use bevy::prelude::*;
use tect_world::world_map::WorldScenePlugin;
use tect_state::app_state::*;

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldScenePlugin)
        .add_plugins(GameStatePlugin)
        .run();
}
