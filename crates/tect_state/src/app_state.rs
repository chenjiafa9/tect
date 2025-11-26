use bevy::{prelude::*};


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


// --- 共享资源和状态定义 ---

/// 鼠标右键的动作判定结果
/// 作为全局资源，用于在相机控制 (模块一) 和角色移动 (模块二) 之间进行互斥。
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Resource)]
pub enum RightMouseAction {
    #[default]
    None,
    PressedJustNow,          // 刚按下，还没决定
    WaitingForDecision,      // 按住中，还在犹豫
    CameraDrag,              // 已经判定为拖动
    CharacterMove,           // 短促点击 → 这一帧要移动角色
}

//游戏共享资源与状态注册插件
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RightMouseAction>()
           .init_state::<AppState>()
           .init_state::<MenuOptions>();
    }
}