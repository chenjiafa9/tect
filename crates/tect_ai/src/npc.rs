use bevy::prelude::*;
use crate::pathfinding::{GridNode, PathfindingGrid};

/// NPC 组件
#[derive(Component, Debug)]
pub struct Npc {
    pub name: String,
    pub move_speed: f32,
    pub current_path: Vec<Vec3>,
    pub path_index: usize,
    pub state: NpcState,
}

impl Default for Npc {
    fn default() -> Self {
        Self {
            name: "NPC".to_string(),
            move_speed: 3.0,
            current_path: Vec::new(),
            path_index: 0,
            state: NpcState::Idle,
        }
    }
}

impl Npc {
    pub fn new(name: impl Into<String>, move_speed: f32) -> Self {
        Self {
            name: name.into(),
            move_speed,
            current_path: Vec::new(),
            path_index: 0,
            state: NpcState::Idle,
        }
    }

    /// 设置新的目标位置并计算路径
    pub fn set_target(&mut self, current_pos: Vec3, target_pos: Vec3, grid: &PathfindingGrid) {
        let start_node = GridNode::from_world(current_pos, grid.grid_size);
        let goal_node = GridNode::from_world(target_pos, grid.grid_size);

        if let Some(path) = grid.find_path(start_node, goal_node) {
            self.current_path = path;
            self.path_index = 0;
            self.state = NpcState::Moving;
            info!("NPC {} 找到路径，共 {} 个节点", self.name, self.current_path.len());
        } else {
            warn!("NPC {} 无法找到到达目标的路径", self.name);
            self.state = NpcState::Idle;
        }
    }

    /// 清除当前路径
    pub fn clear_path(&mut self) {
        self.current_path.clear();
        self.path_index = 0;
        self.state = NpcState::Idle;
    }

    /// 检查是否有有效路径
    pub fn has_path(&self) -> bool {
        !self.current_path.is_empty() && self.path_index < self.current_path.len()
    }

    /// 获取当前目标点
    pub fn current_target(&self) -> Option<Vec3> {
        if self.has_path() {
            Some(self.current_path[self.path_index])
        } else {
            None
        }
    }

    /// 移动到下一个路径点
    pub fn advance_path(&mut self) {
        self.path_index += 1;
        if self.path_index >= self.current_path.len() {
            self.clear_path();
        }
    }
}

/// NPC 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcState {
    Idle,
    Moving,
    Patrolling,
    Following,
    Attacking,
}

/// NPC 巡逻组件
#[derive(Component, Debug)]
pub struct NpcPatrol {
    pub patrol_points: Vec<Vec3>,
    pub current_point_index: usize,
    pub wait_time: f32,
    pub current_wait: f32,
}

impl NpcPatrol {
    pub fn new(patrol_points: Vec<Vec3>, wait_time: f32) -> Self {
        Self {
            patrol_points,
            current_point_index: 0,
            wait_time,
            current_wait: 0.0,
        }
    }

    /// 获取下一个巡逻点
    pub fn next_point(&mut self) -> Option<Vec3> {
        if self.patrol_points.is_empty() {
            return None;
        }

        let point = self.patrol_points[self.current_point_index];
        self.current_point_index = (self.current_point_index + 1) % self.patrol_points.len();
        Some(point)
    }
}

/// NPC 跟随组件
#[derive(Component, Debug)]
pub struct NpcFollow {
    pub target_entity: Entity,
    pub follow_distance: f32,
    pub update_interval: f32,
    pub time_since_update: f32,
}

impl NpcFollow {
    pub fn new(target_entity: Entity, follow_distance: f32) -> Self {
        Self {
            target_entity,
            follow_distance,
            update_interval: 0.5, // 每 0.5 秒更新一次路径
            time_since_update: 0.0,
        }
    }
}

/// NPC 视觉标记（用于调试）
#[derive(Component)]
pub struct NpcVisualMarker;

/// NPC 移动系统
pub fn npc_movement_system(
    mut npc_query: Query<(&mut Transform, &mut Npc), With<Npc>>,
    time: Res<Time>,
) {
    for (mut transform, mut npc) in npc_query.iter_mut() {
        if npc.state != NpcState::Moving || !npc.has_path() {
            continue;
        }

        if let Some(target) = npc.current_target() {
            let current_pos = transform.translation;
            let direction = target - current_pos;
            let distance = direction.length();

            // 到达当前路径点
            if distance < 0.3 {
                npc.advance_path();
                continue;
            }

            // 移动
            let move_vec = direction.normalize() * npc.move_speed * time.delta_secs();
            let look_dir = Vec3::new(move_vec.x, 0.0, move_vec.z).normalize_or_zero();

            // 面向移动方向
            if look_dir.length_squared() > 0.0 {
                transform.look_at(current_pos - look_dir, Vec3::Y);
            }

            // 只移动 XZ 平面
            transform.translation += move_vec.with_y(current_pos.y);
        }
    }
}

/// NPC 巡逻系统
pub fn npc_patrol_system(
    mut npc_query: Query<(&Transform, &mut Npc, &mut NpcPatrol), With<Npc>>,
    grid: Res<PathfindingGrid>,
    time: Res<Time>,
) {
    for (transform, mut npc, mut patrol) in npc_query.iter_mut() {
        // 只有在空闲状态时才开始新的巡逻
        if npc.state != NpcState::Idle && npc.state != NpcState::Patrolling {
            continue;
        }

        // 等待时间
        if patrol.current_wait > 0.0 {
            patrol.current_wait -= time.delta_secs();
            continue;
        }

        // 如果没有路径，设置下一个巡逻点
        if !npc.has_path() {
            if let Some(next_point) = patrol.next_point() {
                npc.set_target(transform.translation, next_point, &grid);
                npc.state = NpcState::Patrolling;
                patrol.current_wait = patrol.wait_time;
            }
        }
    }
}

/// NPC 跟随系统
pub fn npc_follow_system(
    mut npc_query: Query<(&Transform, &mut Npc, &mut NpcFollow), With<Npc>>,
    target_query: Query<&Transform, Without<Npc>>,
    grid: Res<PathfindingGrid>,
    time: Res<Time>,
) {
    for (transform, mut npc, mut follow) in npc_query.iter_mut() {
        if npc.state != NpcState::Following && npc.state != NpcState::Idle {
            continue;
        }

        follow.time_since_update += time.delta_secs();

        // 定期更新路径
        if follow.time_since_update >= follow.update_interval {
            follow.time_since_update = 0.0;

            if let Ok(target_transform) = target_query.get(follow.target_entity) {
                let distance = transform.translation.distance(target_transform.translation);

                // 如果距离太远，重新计算路径
                if distance > follow.follow_distance {
                    npc.set_target(transform.translation, target_transform.translation, &grid);
                    npc.state = NpcState::Following;
                } else {
                    // 距离足够近，停止移动
                    npc.clear_path();
                }
            }
        }
    }
}

/// 可视化 NPC 路径（调试用）
pub fn visualize_npc_paths(
    mut gizmos: Gizmos,
    npc_query: Query<(&Transform, &Npc), With<Npc>>,
) {
    for (transform, npc) in npc_query.iter() {
        if npc.current_path.is_empty() {
            continue;
        }

        // 绘制路径线
        let mut prev_point = transform.translation;
        for point in &npc.current_path {
            gizmos.line(prev_point, *point, bevy::color::palettes::css::YELLOW);
            gizmos.sphere(*point, 0.2, bevy::color::palettes::css::RED);
            prev_point = *point;
        }

        // 高亮当前目标点
        if let Some(current_target) = npc.current_target() {
            gizmos.sphere(current_target, 0.3, bevy::color::palettes::css::GREEN);
        }
    }
}
