//! Challenge 2: an inventory.
//!
//! - A fixed 5x4 grid, some slots holding items, the rest empty.
//! - Each item is an image with its stack count over the corner.
//! - Drag an item onto another slot to swap them. Image and count move with the pointer
//!   (and may leave the inventory); the original slot is empty while dragging.
//! - Hovering an item shows its name in a tooltip.
//!
//! Dragging, the tooltip and the themed panel are library pieces; what a drop *means*
//! (here: swap the two slots' grid cells) is the one game-specific observer.
//!
//! Run with: `cargo run --example inventory`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const COLS: i16 = 5;
const ROWS: i16 = 4;
const SLOT: f32 = 76.0;

/// (name, icon, count, column, row) -- 1-based grid cells.
const ITEMS: &[(&str, &str, u32, i16, i16)] = &[
    ("Sword", "icons/sword.png", 1, 1, 1),
    ("Shield", "icons/shield.png", 1, 2, 1),
    ("Healing potion", "icons/potion.png", 5, 3, 1),
    ("Arrows", "icons/arrow.png", 32, 5, 2),
    ("Gold", "icons/gold.png", 99, 1, 3),
    ("Emerald", "icons/gem.png", 3, 4, 3),
    ("Torch", "icons/torch.png", 8, 2, 4),
    ("Apple", "icons/apple.png", 12, 5, 4),
];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, scene.spawn())
        .run();
}

fn scene() -> impl SceneList {
    let cells = (1..=ROWS).flat_map(|row| (1..=COLS).map(move |col| (col, row)));
    let slots: Vec<_> = cells
        .map(|(col, row)| {
            let item = ITEMS.iter().find(|i| (i.3, i.4) == (col, row)).copied();
            slot(col, row, item)
        })
        .collect();
    bsn_list! {
        Camera2d
        --
        @scenes::ui_root()
        Children [
            @scenes::card()
            Children [
                @scenes::header("Inventory")
                --
                @scenes::grid(COLS as u16, 6.0)
                Children [ {slots} ]
            ]
        ]
    }
}

/// One cell. Every cell accepts drops; only full ones can be picked up.
fn slot(col: i16, row: i16, item: Option<(&'static str, &'static str, u32, i16, i16)>) -> impl Scene {
    let item_parts = item.map(|(name, _, _, _, _)| {
        bsn! { @scenes::draggable() @scenes::tooltip(name) }
    });
    let contents = item.map(|(_, icon, count, _, _)| bsn_list! { @item_icon(icon, count) });
    bsn! {
        Node {
            width: {px(SLOT)},
            height: {px(SLOT)},
            grid_column: {GridPlacement::start(col)},
            grid_row: {GridPlacement::start(row)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(6.0))},
        }
        ThemedBackground(ColorToken::InputBg)
        ThemedBorderColor(ColorToken::InputBorder)
        @scenes::drop_target()
        @{item_parts}
        on(|drop: On<Dropped>, mut nodes: Query<&mut Node>| {
            if let Ok([mut a, mut b]) = nodes.get_many_mut([drop.entity, drop.dragged]) {
                core::mem::swap(&mut a.grid_column, &mut b.grid_column);
                core::mem::swap(&mut a.grid_row, &mut b.grid_row);
            }
        })
        Children [ {contents} ]
    }
}

/// The item picture with its stack count in the bottom-right corner.
fn item_icon(icon: &'static str, count: u32) -> impl Scene {
    let count = if count > 1 { count.to_string() } else { String::new() };
    bsn! {
        @scenes::icon(icon, SLOT - 18.0)
        Node { justify_content: JustifyContent::FlexEnd, align_items: AlignItems::FlexEnd }
        Pickable::IGNORE
        Children [
            @scenes::label(count) ThemedFont(FontToken::Small) TextLayout::no_wrap() Pickable::IGNORE
        ]
    }
}
