use bevy::prelude::*;


//游戏主状态
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
}

//主菜单界面子选项页
// "NEW GEANE", "CONTINUE GAME", "ONLINE GAME", "SETING", "ABOUT"
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::Menu)]
#[states(scoped_entities)]
pub enum MenuOptions {
    #[default]
    NewGame,
    ContinueGame, 
    OnlineGame,
    Setting,
    About
}