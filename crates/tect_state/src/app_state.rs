use bevy::{prelude::*, time::Stopwatch};


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
    /// 右键未按下，或动作已完成。
    #[default]
    None,
    /// 右键刚按下，正在等待拖动或点击的判定。
    AwaitingDecision,
    /// 判定为拖动操作（由相机系统处理）。
    CameraDrag,
    /// 判定为点击操作（由角色移动系统处理）。
    CharacterMove,
}

/// 用于在 `AwaitingDecision` 状态下计时的资源。
/// 必须作为资源存在，因为需要在多个系统和帧之间共享计时状态。
#[derive(Resource)]
pub struct DragDecisionTimer(pub Stopwatch);

impl Default for DragDecisionTimer {
    fn default() -> Self {
        DragDecisionTimer(Stopwatch::new())
    }
}


//游戏共享资源与状态注册插件
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RightMouseAction>()
           .init_resource::<DragDecisionTimer>()
           .init_state::<AppState>()
           .init_state::<MenuOptions>();
    }
}