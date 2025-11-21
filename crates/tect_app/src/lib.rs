use bevy::prelude::*;
use tect_world::world_map::WorldScenePlugin;


pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WorldScenePlugin)
        .run();
}
