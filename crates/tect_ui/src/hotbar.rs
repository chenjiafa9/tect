// src/ui/hotbar.rs
//! 底部快捷工具栏（Hotbar）
//! 使用组件元组（无任何 Bundle）
//! 键盘 1~0 切换槽位，高亮 + 联动工具/可放置物

use bevy::prelude::*;
use tect_assetload::asset_load::GameAssets;
use tect_control::object_interaction::{PlayerTool, SelectedPlaceable};
use tect_state::app_state::*;

pub struct HotbarPlugin;

impl Plugin for HotbarPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentHotbarSelection>()
            .add_systems(OnEnter(AppState::InGame), spawn_hotbar)
            .add_systems(
                Update,
                (hotbar_keyboard_system, hotbar_highlight_and_link_system)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

// ────────────────────── 资源：当前选中槽位 ──────────────────────
#[derive(Resource, Default)]
pub struct CurrentHotbarSelection {
    pub index: usize, // 0~9
}

// ────────────────────── 组件：快捷栏槽位 ──────────────────────
#[derive(Component)]
pub struct HotbarSlot {
    pub index: usize,
}

// ────────────────────── 生成工具栏 UI ──────────────────────
fn spawn_hotbar(mut commands: Commands, assets: Res<GameAssets>) {
    let slot_size_px = 50.0;
    let slot_margin_px = 4.0; // margin
    let slot_border_px = 3.0; // border
    let padding_px = 4.0; // 根节点 padding

    // 计算总宽度（10 个槽位 + margin + padding）
    let single_slot_width = slot_size_px + slot_margin_px * 2.0 + slot_border_px * 2.0;
    let total_slots_width = single_slot_width * 10.0;
    let total_padding = padding_px * 2.0;
    let total_width = total_slots_width + total_padding;

    // 根节点（背景面板）
    commands
        .spawn((
            Node {
                border: UiRect::all(px(2.0)),
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                left: Val::Percent(20.0),
                padding: UiRect::all(Val::Px(padding_px)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            Transform::from_translation(Vec3::new(-total_width / 2.0, 0.0, 0.0)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            BorderColor::all(Color::WHITE),
            Name::new("Hotbar Root"),
            Pickable::IGNORE,
            PickingBlocker,
        ))
        .with_children(|parent| {
            for i in 0..10 {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(slot_size_px),
                            height: Val::Px(slot_size_px),
                            margin: UiRect::all(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(3.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderColor::all(Color::WHITE.with_alpha(0.3)),
                        HotbarSlot { index: i },
                        Name::new(format!("Hotbar Slot {}", i + 1)),
                    ))
                    .with_children(|p| {
                        p.spawn(Sprite::from_image(assets.ui_placeholder_icon.clone()));
                        // 替换图标资源
                    });

                // 槽位内图标（占位，你可以替换为真实物品图标）
            }
        });
}

// ────────────────────── 键盘输入：1~0 切换槽位 ──────────────────────
fn hotbar_keyboard_system(
    mut selection: ResMut<CurrentHotbarSelection>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let key_map = [
        (KeyCode::Digit1, 0),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
        (KeyCode::Digit5, 4),
        (KeyCode::Digit6, 5),
        (KeyCode::Digit7, 6),
        (KeyCode::Digit8, 7),
        (KeyCode::Digit9, 8),
        (KeyCode::Digit0, 9),
    ];

    for (key, index) in key_map {
        if keys.just_pressed(key) {
            selection.index = index;
            info!("快捷栏切换 → 槽位 {}", index + 1);
        }
    }
}

// ────────────────────── 高亮 + 联动工具/可放置物 ──────────────────────
fn hotbar_highlight_and_link_system(
    mut slots: Query<(&HotbarSlot, &mut BorderColor, &mut BackgroundColor)>,
    selection: Res<CurrentHotbarSelection>,
    mut player_tool: Query<&mut PlayerTool>,
    mut selected_placeable: ResMut<SelectedPlaceable>,
    assets: Res<GameAssets>,
) {
    if !selection.is_changed() | player_tool.is_empty() {
        return;
    }

    let mut tool = player_tool.single_mut().expect("获取工具失败！");
    *selected_placeable = SelectedPlaceable::default(); // 默认清空

    // 示例联动逻辑（根据你的物品表自定义）
    match selection.index {
        0 => *tool = PlayerTool::Axe, // 槽位 1：斧头
        1 => {
            *tool = PlayerTool::Axe;
            *selected_placeable = SelectedPlaceable {
                scene: assets.player_scene.clone(), // 你的树模型
                name: "Oak Tree".to_string(),
            };
        }
        2 => {
            *tool = PlayerTool::Shovel;
            // 挖土工具，不放置
        }
        // 其他槽位继续扩展...
        _ => *tool = PlayerTool::None,
    }

    // 高亮当前选中槽位
    for (slot, mut border, mut bg) in &mut slots {
        if slot.index == selection.index {
            *border = BorderColor::all(Color::srgb(1.0, 1.0, 0.0));
            *bg = BackgroundColor(Color::srgba(0.3, 0.3, 0.1, 0.8));
        } else {
            *border = BorderColor::all(Color::WHITE.with_alpha(0.3));
            *bg = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8));
        }
    }
}
