use bevy::prelude::*;

/// ========================
/// 1. 玩家动画状态（推荐用这个驱动动画）
/// ========================
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerAnimationState {
    #[default]
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
    AttackSwing,
    AttackThrust,
    Mining,
    Eating,
    Drinking,
    Death,
    // 你可以继续加：Swim, Climb, Dance, Emote1...
}

impl PlayerAnimationState {
    pub fn is_moving(&self) -> bool {
        matches!(self, PlayerAnimationState::Walk | PlayerAnimationState::Run)
    }

    pub fn is_alive(&self) -> bool {
        !matches!(self, PlayerAnimationState::Death)
    }
}

// /// 基础角色属性
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, FromReflect, Serialize, Deserialize)]
// pub struct Attributes {
//     pub strength: i32,
//     pub agility: i32,
//     pub intelligence: i32,
//     pub endurance: i32,
//     pub perception: i32,
//     pub luck: i32,
// }

// impl Default for Attributes {
//     fn default() -> Self {
//         Self {
//             strength: 5,
//             agility: 5,
//             intelligence: 5,
//             endurance: 5,
//             perception: 5,
//             luck: 5,
//         }
//     }
// }

// /// 状态效果（buff/debuff）
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct StatusEffect {
//     pub id: String,                 // 唯一标识
//     pub remaining: Duration,        // 剩余时间
//     pub stacks: u8,                 // 层数
//     pub source: Option<String>,     // 来源（可选）
//     // TODO: 可以扩展为 effect type, modifiers, tick callbacks 等
// }

// impl StatusEffect {
//     pub fn tick(&mut self, delta: Duration) {
//         if delta >= self.remaining {
//             self.remaining = Duration::ZERO;
//         } else {
//             self.remaining -= delta;
//         }
//     }
// }

/// ========================
/// 2. 玩家核心属性
/// ========================
#[derive(Component, Debug)]
pub struct PlayerStats {
    // 基础标识
    pub name: String,
    pub level: u32,
    pub xp: u64,
    pub xp_to_next: u64,

    // 生命/资源
    pub hp: f32,
    pub max_hp: f32,
    pub mana: f32, // 或魔法值/能量
    pub max_mana: f32,
    pub stamina: f32, // 体力 - 用于奔跑/跳跃/攻击等
    pub max_stamina: f32,

    // 生存类（沙盒常用）
    pub hunger: f32, // 0..=100, 0 = 饥饿（或你可反向定义）
    pub max_hunger: f32,
    pub thirst: f32, // 口渴
    pub max_thirst: f32,
    pub body_temperature: f32, // 体温（可用于环境伤害）

    // 战斗/属性相关
    pub armor: f32,
    pub attack_power: f32,
    pub move_speed: f32, // 当前移动速度（可受状态影响）
    pub base_move_speed: f32,

    // 负重与背包（仅字段示例）
    pub carry_weight: f32,
    pub carry_capacity: f32,

    // 抗性/减伤（示例）
    pub physical_resist: f32,
    pub fire_resist: f32,
    pub cold_resist: f32,
    pub poison_resist: f32,

    // 策略数值与玩法扩展
    pub skill_points: u32,
    // pub attributes: Attributes,

    // 状态效果
    // pub status_effects: Vec<StatusEffect>,

    // 更新时间戳或冷却（可扩展）
    pub last_regen: Option<std::time::Instant>,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            name: "Player".to_string(),
            level: 1,
            xp: 0,
            xp_to_next: 100,
            hp: 100.0,
            max_hp: 100.0,
            mana: 50.0,
            max_mana: 50.0,
            stamina: 100.0,
            max_stamina: 100.0,
            hunger: 100.0,
            max_hunger: 100.0,
            thirst: 100.0,
            max_thirst: 100.0,
            body_temperature: 37.0,
            armor: 0.0,
            attack_power: 10.0,
            move_speed: 5.0,
            base_move_speed: 5.0,
            carry_weight: 0.0,
            carry_capacity: 50.0,
            physical_resist: 0.0,
            fire_resist: 0.0,
            cold_resist: 0.0,
            poison_resist: 0.0,
            skill_points: 0,
            // attributes: Attributes::default(),
            // status_effects: Vec::new(),
            last_regen: None,
        }
    }
}

impl PlayerStats {
    pub fn new(name: impl Into<String>, level: u32) -> Self {
        let mut s = Self::default();
        s.name = name.into();
        s.level = level;
        // 简单依据等级放大血量/资源
        s.max_hp = 100.0 + (level.saturating_sub(1) as f32) * 10.0;
        s.hp = s.max_hp;
        s.max_mana = 50.0 + (level.saturating_sub(1) as f32) * 5.0;
        s.mana = s.max_mana;
        s.max_stamina = 100.0;
        s.stamina = s.max_stamina;
        s.xp_to_next = 100 + ((level as u64) * 50);
        s
    }

    /// 增加经验并尝试升级
    pub fn add_xp(&mut self, amount: u64) -> bool {
        self.xp = self.xp.saturating_add(amount);
        let mut leveled = false;
        while self.xp >= self.xp_to_next {
            self.xp -= self.xp_to_next;
            self.level += 1;
            self.skill_points += 1;
            // 升级收益（示例）
            self.max_hp += 10.0;
            self.max_mana += 5.0;
            self.attack_power += 1.0;
            self.xp_to_next = 100 + ((self.level as u64) * 50);
            leveled = true;
        }
        if leveled {
            // 恢复到满（可自定义）
            self.hp = self.max_hp;
            self.mana = self.max_mana;
            self.stamina = self.max_stamina;
        }
        leveled
    }

    /// 受到伤害（考虑护甲/抗性的逻辑可在外部封装或在此扩展）
    pub fn take_damage(&mut self, amount: f32) -> bool {
        let mitigated = (amount - self.armor).max(0.0);
        self.hp = (self.hp - mitigated).max(0.0);
        self.hp <= 0.0
    }

    pub fn heal(&mut self, amount: f32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }

    pub fn consume_stamina(&mut self, amount: f32) -> bool {
        if amount <= 0.0 {
            return true;
        }
        if self.stamina >= amount {
            self.stamina -= amount;
            true
        } else {
            self.stamina = 0.0;
            false
        }
    }

    pub fn change_hunger(&mut self, delta: f32) {
        self.hunger = (self.hunger + delta).clamp(0.0, self.max_hunger);
    }

    pub fn change_thirst(&mut self, delta: f32) {
        self.thirst = (self.thirst + delta).clamp(0.0, self.max_thirst);
    }

    /// 每帧/每秒恢复处理（简化示例）
    /// - delta_seconds: 本次 tick 的秒数
    pub fn regen_tick(&mut self, delta_seconds: f32) {
        // 状态效果影响可在此合并
        // 饥饿/口渴随时间下降
        let hunger_decay = 0.1 * delta_seconds; // 每秒掉 0.1 饥饿值
        let thirst_decay = 0.15 * delta_seconds;
        self.change_hunger(-hunger_decay);
        self.change_thirst(-thirst_decay);

        // 若饥饿或口渴值低，则体力与生命自然恢复减慢或反向
        let hunger_factor = (self.hunger / self.max_hunger).clamp(0.0, 1.0);
        let thirst_factor = (self.thirst / self.max_thirst).clamp(0.0, 1.0);
        let survival_factor = hunger_factor.min(thirst_factor);

        // 体力自动恢复
        let stamina_regen_rate = 5.0 * survival_factor; // 每秒回复
        self.stamina = (self.stamina + stamina_regen_rate * delta_seconds).min(self.max_stamina);

        // 生命回复（只有在食物/饮水足够时才回复）
        if survival_factor > 0.5 {
            let hp_regen_rate = 0.5 * survival_factor; // 每秒
            self.hp = (self.hp + hp_regen_rate * delta_seconds).min(self.max_hp);
        } else {
            // 极端饥饿会损失生命
            let hp_loss_rate = 0.25 * (1.0 - survival_factor);
            self.hp = (self.hp - hp_loss_rate * delta_seconds).max(0.0);
        }

        // mana 简单回复
        let mana_regen = 2.0 * delta_seconds;
        self.mana = (self.mana + mana_regen).min(self.max_mana);

        // 状态效果计时
        // let mut i = 0;
        // while i < self.status_effects.len() {
        //     let mut eff = self.status_effects.remove(i);
        //     eff.tick(Duration::from_secs_f32(delta_seconds));
        //     if eff.remaining > Duration::ZERO {
        //         self.status_effects.insert(i, eff);
        //         i += 1;
        //     } else {
        //         // effect expired -> do cleanup if needed
        //     }
        // }
    }
}

/// ========================
/// 3. 玩家动画资源（推荐方式：语义化 + 类型安全）
/// ========================
#[derive(Resource, Default, Clone)]
pub struct PlayerAnimations {
    pub graph: Handle<AnimationGraph>,

    // 所有动画的索引（名字清晰，永不混乱）
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub run: AnimationNodeIndex,
    pub jump: AnimationNodeIndex,
    pub attack: AnimationNodeIndex,
    pub mining: AnimationNodeIndex,
    pub eating: AnimationNodeIndex,
    pub death: AnimationNodeIndex,
}

impl PlayerAnimations {
    // 超好用！一行切换动画
    pub fn play(&self, player: &mut AnimationPlayer, state: PlayerAnimationState) {
        let node = match state {
            PlayerAnimationState::Idle => self.idle,
            PlayerAnimationState::Walk => self.walk,
            PlayerAnimationState::Run => self.run,
            PlayerAnimationState::Jump => self.jump,
            PlayerAnimationState::AttackSwing | PlayerAnimationState::AttackThrust => self.attack,
            PlayerAnimationState::Mining => self.mining,
            PlayerAnimationState::Eating => self.eating,
            PlayerAnimationState::Death => self.death,
            _ => return,
        };

        if !player.is_playing_animation(node) {
            let mut playback = player.play(node);
            if state == PlayerAnimationState::Death {
                playback = playback.pause(); // 死亡动画播完就停
            } else if matches!(
                state,
                PlayerAnimationState::Walk | PlayerAnimationState::Run | PlayerAnimationState::Idle
            ) {
                playback = playback.repeat();
            }
            playback.set_speed(1.0);
        }
    }
}
