pub mod pathfinding;
pub mod npc;

use bevy::prelude::*;
use pathfinding::PathfindingGrid;
use npc::{npc_movement_system, npc_patrol_system, npc_follow_system, visualize_npc_paths};

/// AI 插件
pub struct TectAiPlugin;

impl Plugin for TectAiPlugin {
    fn build(&self, app: &mut App) {
        app
            // 初始化寻路网格资源
            .insert_resource(PathfindingGrid::default())
            // 添加 NPC 相关系统
            .add_systems(Update, (
                npc_movement_system,
                npc_patrol_system,
                npc_follow_system,
                visualize_npc_paths,
            ).chain());
    }
}
