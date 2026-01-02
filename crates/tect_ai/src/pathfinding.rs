use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// A* 寻路算法的网格节点
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridNode {
    pub x: i32,
    pub z: i32,
}

impl GridNode {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// 从世界坐标转换为网格坐标
    pub fn from_world(pos: Vec3, grid_size: f32) -> Self {
        Self {
            x: (pos.x / grid_size).round() as i32,
            z: (pos.z / grid_size).round() as i32,
        }
    }

    /// 转换为世界坐标
    pub fn to_world(&self, grid_size: f32) -> Vec3 {
        Vec3::new(self.x as f32 * grid_size, 0.0, self.z as f32 * grid_size)
    }

    /// 计算曼哈顿距离
    pub fn manhattan_distance(&self, other: &GridNode) -> i32 {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }

    /// 计算欧几里得距离（用于更精确的启发式）
    pub fn euclidean_distance(&self, other: &GridNode) -> f32 {
        let dx = (self.x - other.x) as f32;
        let dz = (self.z - other.z) as f32;
        (dx * dx + dz * dz).sqrt()
    }

    /// 获取相邻节点（8方向）
    pub fn neighbors(&self) -> Vec<GridNode> {
        vec![
            GridNode::new(self.x + 1, self.z),     // 右
            GridNode::new(self.x - 1, self.z),     // 左
            GridNode::new(self.x, self.z + 1),     // 上
            GridNode::new(self.x, self.z - 1),     // 下
            GridNode::new(self.x + 1, self.z + 1), // 右上
            GridNode::new(self.x + 1, self.z - 1), // 右下
            GridNode::new(self.x - 1, self.z + 1), // 左上
            GridNode::new(self.x - 1, self.z - 1), // 左下
        ]
    }

    /// 计算移动到相邻节点的代价（对角线移动代价更高）
    pub fn cost_to(&self, other: &GridNode) -> f32 {
        let dx = (self.x - other.x).abs();
        let dz = (self.z - other.z).abs();
        if dx + dz == 2 {
            1.414 // 对角线移动 (sqrt(2))
        } else {
            1.0 // 直线移动
        }
    }
}

/// A* 算法中的节点（用于优先队列）
#[derive(Debug, Clone)]
struct AStarNode {
    node: GridNode,
    g_cost: f32, // 从起点到当前节点的实际代价
    h_cost: f32, // 从当前节点到终点的启发式代价
    f_cost: f32, // g_cost + h_cost
}

impl AStarNode {
    fn new(node: GridNode, g_cost: f32, h_cost: f32) -> Self {
        Self {
            node,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
        }
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 注意：BinaryHeap 是最大堆，我们需要最小堆，所以反转比较
        other.f_cost.partial_cmp(&self.f_cost).unwrap_or(Ordering::Equal)
    }
}

/// 寻路网格配置
#[derive(Resource, Clone)]
pub struct PathfindingGrid {
    pub grid_size: f32,
    pub obstacles: HashSet<GridNode>,
    pub world_bounds: (Vec2, Vec2), // (min, max)
}

impl Default for PathfindingGrid {
    fn default() -> Self {
        Self {
            grid_size: 1.0,
            obstacles: HashSet::new(),
            world_bounds: (Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)),
        }
    }
}

impl PathfindingGrid {
    /// 创建新的寻路网格
    pub fn new(grid_size: f32, world_bounds: (Vec2, Vec2)) -> Self {
        Self {
            grid_size,
            obstacles: HashSet::new(),
            world_bounds,
        }
    }

    /// 添加障碍物
    pub fn add_obstacle(&mut self, node: GridNode) {
        self.obstacles.insert(node);
    }

    /// 移除障碍物
    pub fn remove_obstacle(&mut self, node: GridNode) {
        self.obstacles.remove(&node);
    }

    /// 检查节点是否可通行
    pub fn is_walkable(&self, node: &GridNode) -> bool {
        // 检查是否在世界边界内
        let world_pos = node.to_world(self.grid_size);
        if world_pos.x < self.world_bounds.0.x
            || world_pos.x > self.world_bounds.1.x
            || world_pos.z < self.world_bounds.0.y
            || world_pos.z > self.world_bounds.1.y
        {
            return false;
        }

        // 检查是否是障碍物
        !self.obstacles.contains(node)
    }

    /// A* 寻路算法
    pub fn find_path(&self, start: GridNode, goal: GridNode) -> Option<Vec<Vec3>> {
        if !self.is_walkable(&start) || !self.is_walkable(&goal) {
            return None;
        }

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<GridNode, GridNode> = HashMap::new();
        let mut g_scores: HashMap<GridNode, f32> = HashMap::new();
        let mut closed_set: HashSet<GridNode> = HashSet::new();

        g_scores.insert(start, 0.0);
        let h_cost = start.euclidean_distance(&goal);
        open_set.push(AStarNode::new(start, 0.0, h_cost));

        while let Some(current) = open_set.pop() {
            if current.node == goal {
                // 重建路径
                return Some(self.reconstruct_path(&came_from, current.node));
            }

            if closed_set.contains(&current.node) {
                continue;
            }
            closed_set.insert(current.node);

            // 检查所有邻居
            for neighbor in current.node.neighbors() {
                if !self.is_walkable(&neighbor) || closed_set.contains(&neighbor) {
                    continue;
                }

                let tentative_g_score = current.g_cost + current.node.cost_to(&neighbor);
                let neighbor_g_score = *g_scores.get(&neighbor).unwrap_or(&f32::INFINITY);

                if tentative_g_score < neighbor_g_score {
                    came_from.insert(neighbor, current.node);
                    g_scores.insert(neighbor, tentative_g_score);
                    let h_cost = neighbor.euclidean_distance(&goal);
                    open_set.push(AStarNode::new(neighbor, tentative_g_score, h_cost));
                }
            }
        }

        None // 没有找到路径
    }

    /// 重建路径
    fn reconstruct_path(&self, came_from: &HashMap<GridNode, GridNode>, mut current: GridNode) -> Vec<Vec3> {
        let mut path = vec![current.to_world(self.grid_size)];
        while let Some(&prev) = came_from.get(&current) {
            current = prev;
            path.push(current.to_world(self.grid_size));
        }
        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_node_conversion() {
        let grid_size = 1.0;
        let world_pos = Vec3::new(5.0, 0.0, 3.0);
        let node = GridNode::from_world(world_pos, grid_size);
        assert_eq!(node.x, 5);
        assert_eq!(node.z, 3);

        let converted = node.to_world(grid_size);
        assert!((converted.x - 5.0).abs() < 0.01);
        assert!((converted.z - 3.0).abs() < 0.01);
    }

    #[test]
    fn test_pathfinding() {
        let mut grid = PathfindingGrid::new(1.0, (Vec2::new(-10.0, -10.0), Vec2::new(10.0, 10.0)));
        
        // 添加一些障碍物
        grid.add_obstacle(GridNode::new(1, 0));
        grid.add_obstacle(GridNode::new(1, 1));
        grid.add_obstacle(GridNode::new(1, 2));

        let start = GridNode::new(0, 1);
        let goal = GridNode::new(3, 1);

        let path = grid.find_path(start, goal);
        assert!(path.is_some());
        
        if let Some(p) = path {
            assert!(p.len() > 0);
            println!("Path found with {} nodes", p.len());
        }
    }
}
