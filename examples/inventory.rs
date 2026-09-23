//! Inventory example for `lava_ui_builder` — Challenge 2, on the scene (BSN) API.
//!
//! Demonstrates a drag-and-drop inventory grid using Bevy's picking system.
//!   - Fixed 5×4 grid; some slots contain items, others are empty.
//!   - Each item shows a colored icon + stack count in the corner.
//!   - Dragging moves the item visually (`UiTransform` offset); the layout slot
//!     remains reserved but the item renders at the cursor position.
//!   - Dropping on another slot swaps the two slots' grid positions.
//!   - Hovering an item shows a tooltip with its name.
//!
//! The interesting part for BSN is `slot()`: six observers attached inline with `on(..)`,
//! next to the entity they act on, instead of six `.observe()` calls after a spawn. Note
//! that `on(..)` needs a `Clone` system, which is why the captured item name is a
//! `&'static str` rather than a `String`.
//!
//! Run with: `cargo run --example inventory`

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::{scenes, LavaTheme, LavaUiPlugin};

const COLS: i16 = 5;
const ROWS: i16 = 4;
const SLOT_SIZE: f32 = 84.0;
const GAP: f32 = 6.0;

// (name, count, color_rgb, col, row) — col/row are 1-based
type ItemDef = (&'static str, u32, (f32, f32, f32), i16, i16);
const ITEMS: &[ItemDef] = &[
    ("Sword",  1,  (0.75, 0.20, 0.20), 1, 1),
    ("Shield", 1,  (0.20, 0.40, 0.80), 2, 1),
    ("Potion", 5,  (0.70, 0.20, 0.65), 3, 1),
    ("Arrow",  32, (0.60, 0.40, 0.20), 5, 2),
    ("Gold",   99, (0.90, 0.80, 0.10), 1, 3),
    ("Gem",    3,  (0.30, 0.90, 0.70), 4, 3),
    ("Torch",  8,  (1.00, 0.60, 0.10), 2, 4),
];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, scene.spawn())
        .add_systems(Update, position_tooltip)
        .run();
}

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component, Default, Clone)]
struct InventoryItem;

#[derive(Component, Default, Clone)]
struct TooltipRoot;
#[derive(Component, Default, Clone)]
struct TooltipText;

type Item = Option<(&'static str, u32, Color)>;

// ── Scene ─────────────────────────────────────────────────────────────────────

fn scene() -> impl SceneList {
    bsn_list![Camera2d, tooltip(), root()]
}

/// An absolutely-positioned overlay that follows the cursor, hidden until a hover.
fn tooltip() -> impl Scene {
    bsn! {
        TooltipRoot
        Node { position_type: PositionType::Absolute, padding: {UiRect::all(px(8.0))} }
        BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.92))
        BorderColor::all(Color::srgb(0.6, 0.6, 0.85))
        GlobalZIndex(200)
        Visibility::Hidden
        Children [(
            TooltipText
            scenes::text("", 15.0, Color::WHITE)
            Pickable::IGNORE
        )]
    }
}

fn root() -> impl Scene {
    let mut item_map: HashMap<(i16, i16), (&'static str, u32, Color)> = HashMap::default();
    for &(name, count, (r, g, b), col, row) in ITEMS {
        item_map.insert((col, row), (name, count, Color::srgb(r, g, b)));
    }

    let slots: Vec<_> = (1..=ROWS)
        .flat_map(|row| (1..=COLS).map(move |col| (col, row)))
        .map(|(col, row)| slot(col, row, item_map.get(&(col, row)).copied()))
        .collect();

    bsn! {
        scenes::ui_root()
        Node { row_gap: {px(16.0)} }
        Children [
            scenes::text("Inventory", 36.0, Color::WHITE),
            (
                Node {
                    display: Display::Grid,
                    column_gap: {px(GAP)},
                    row_gap: {px(GAP)},
                    padding: {UiRect::all(px(12.0))},
                }
                BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.96))
                BorderColor::all(Color::srgb(0.28, 0.28, 0.52))
                Pickable::IGNORE
                Children [ {slots} ]
            ),
        ]
    }
}

/// One inventory slot, with every drag/hover observer attached inline.
fn slot(col: i16, row: i16, item: Item) -> impl Scene {
    let item_name = item.map(|(n, _, _)| n);
    // `Option<impl Scene>` is a Scene: present only when the slot holds something.
    let marker = item.is_some().then(|| bsn! { InventoryItem });
    // `Option<impl SceneList>` for the child, because a `Children [..]` slot takes a list.
    let icon = item.map(|(_, count, color)| bsn_list![item_icon(count, color)]);

    bsn! {
        Node {
            width: {px(SLOT_SIZE)},
            height: {px(SLOT_SIZE)},
            grid_row: {GridPlacement::start(row)},
            grid_column: {GridPlacement::start(col)},
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: {UiRect::all(px(2.0))},
        }
        BackgroundColor(Color::srgb(0.10, 0.10, 0.18))
        BorderColor::all(Color::srgb(0.24, 0.24, 0.44))
        Outline { width: {px(3.0)}, offset: {Val::ZERO}, color: {Color::NONE} }
        GlobalZIndex
        // should_block_lower: false lets drag events reach the slot underneath
        Pickable { should_block_lower: false, is_hoverable: true }
        {marker}

        // Hover -> show the tooltip with this item's name.
        on(move |ev: On<Pointer<Over>>,
                 items: Query<&InventoryItem>,
                 mut vis_q: Query<&mut Visibility, With<TooltipRoot>>,
                 mut txt_q: Query<&mut Text, With<TooltipText>>| {
            if items.get(ev.event_target()).is_ok() {
                if let Ok(mut v) = vis_q.single_mut() {
                    *v = Visibility::Visible;
                }
                if let Ok(mut t) = txt_q.single_mut() {
                    **t = item_name.unwrap_or("").to_string();
                }
            }
        })
        on(|_: On<Pointer<Out>>, mut vis_q: Query<&mut Visibility, With<TooltipRoot>>| {
            if let Ok(mut v) = vis_q.single_mut() {
                *v = Visibility::Hidden;
            }
        })

        // Drag start -> white outline, lifted above the other slots.
        on(|ev: On<Pointer<DragStart>>, mut q: Query<(&mut Outline, &mut GlobalZIndex)>| {
            if let Ok((mut outline, mut zidx)) = q.get_mut(ev.event_target()) {
                outline.color = Color::WHITE;
                zidx.0 = 10;
            }
        })
        // Drag -> offset the visual position; the layout slot stays reserved.
        on(|ev: On<Pointer<Drag>>, mut q: Query<&mut UiTransform>| {
            if let Ok(mut t) = q.get_mut(ev.event_target()) {
                t.translation = Val2::px(ev.distance.x, ev.distance.y);
            }
        })
        on(|ev: On<Pointer<DragEnd>>,
            mut q: Query<(&mut UiTransform, &mut Outline, &mut GlobalZIndex)>| {
            if let Ok((mut t, mut outline, mut zidx)) = q.get_mut(ev.event_target()) {
                t.translation = Val2::ZERO;
                outline.color = Color::NONE;
                zidx.0 = 0;
            }
        })
        // Drop -> swap grid positions, so the two items exchange slots.
        on(|ev: On<Pointer<DragDrop>>, mut q: Query<&mut Node>| {
            if let Ok([mut a, mut b]) = q.get_many_mut([ev.event_target(), ev.dropped]) {
                core::mem::swap(&mut a.grid_row, &mut b.grid_row);
                core::mem::swap(&mut a.grid_column, &mut b.grid_column);
            }
        })

        Children [ {icon} ]
    }
}

/// The coloured square, with the stack count tucked into its corner.
fn item_icon(count: u32, color: Color) -> impl Scene {
    let count_text = if count > 1 {
        count.to_string()
    } else {
        String::new()
    };
    bsn! {
        Node {
            width: {px(SLOT_SIZE - 16.0)},
            height: {px(SLOT_SIZE - 16.0)},
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            padding: {UiRect::all(px(3.0))},
        }
        BackgroundColor(color)
        Pickable::IGNORE
        Children [(
            scenes::text(count_text, 14.0, Color::WHITE)
            Pickable::IGNORE
        )]
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Keep the tooltip anchored to the cursor position.
fn position_tooltip(windows: Query<&Window>, mut q: Query<&mut Node, With<TooltipRoot>>) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    if let Ok(mut node) = q.single_mut() {
        node.left = Val::Px(cursor.x + 14.0);
        node.top = Val::Px(cursor.y + 14.0);
    }
}
