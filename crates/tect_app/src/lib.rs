use bevy::prelude::*;
use tect_ui::main_ui::MainUiPlugin;

pub fn run() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(MainUiPlugin)
    .run();
}
