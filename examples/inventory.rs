//! Inventory example for `lava_ui_builder` — Challenge 2.
//!
//! Demonstrates a drag-and-drop inventory grid using Bevy's picking system.
//!   - Fixed 5×4 grid; some slots contain items, others are empty.
//!   - Each item shows a colored icon + stack count in the corner.
//!   - Dragging moves the item visually (UiTransform offset); the layout slot
//!     remains reserved but the item renders at the cursor position.
//!   - Dropping on another slot swaps the two slots' grid positions.
//!   - Items can be dragged outside the inventory bounding box.
//!   - Hovering an item shows a tooltip with its name.
//!
//! Run with: `cargo run --example inventory`

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::*;

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
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_plugins(LavaUiPlugin)
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(Update, position_tooltip)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Marker components ─────────────────────────────────────────────────────────

#[derive(Component)]
struct InventoryItem;

#[derive(Component)] struct TooltipRoot;
#[derive(Component)] struct TooltipText;

type Item = Option<(&'static str, u32, Color)>;

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_ui(mut commands: Commands) {
    // Tooltip — absolute overlay, tracks cursor, hidden until hover
    commands.spawn((
        TooltipRoot,
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.92)),
        BorderColor::all(Color::srgb(0.6, 0.6, 0.85)),
        GlobalZIndex(200),
        Visibility::Hidden,
    )).with_child((
        TooltipText,
        Text::new(""),
        TextFont::default().with_font_size(15.0),
        TextColor(Color::WHITE),
        Pickable::IGNORE,
    ));

    // Build item lookup: (col, row) → item data
    let mut item_map: HashMap<(i16, i16), (&'static str, u32, Color)> = HashMap::default();
    for &(name, count, (r, g, b), col, row) in ITEMS {
        item_map.insert((col, row), (name, count, Color::srgb(r, g, b)));
    }

    // Spawn all slots, collecting their entity IDs
    let mut slot_ids = Vec::new();
    for row in 1..=ROWS {
        for col in 1..=COLS {
            let item = item_map.get(&(col, row)).copied();
            slot_ids.push(spawn_slot(&mut commands, col, row, item));
        }
    }

    // Inventory grid container
    let grid = commands.spawn((
        Node {
            display: Display::Grid,
            column_gap: Val::Px(GAP),
            row_gap: Val::Px(GAP),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.96)),
        BorderColor::all(Color::srgb(0.28, 0.28, 0.52)),
        Pickable::IGNORE,
    )).add_children(&slot_ids).id();

    // Title
    let title = commands.spawn((
        Text::new("Inventory"),
        TextFont::default().with_font_size(36.0),
        TextColor(Color::WHITE),
    )).id();

    // Full-screen root (centers everything)
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        Pickable::IGNORE,
    )).add_children(&[title, grid]);
}

/// Spawn one inventory slot with all drag/hover observers attached.
/// Returns the slot's entity ID so it can be added to the grid.
fn spawn_slot(commands: &mut Commands, col: i16, row: i16, item: Item) -> Entity {
    let item_name = item.map(|(n, _, _)| n);

    let mut slot = commands.spawn((
        Node {
            width: Val::Px(SLOT_SIZE),
            height: Val::Px(SLOT_SIZE),
            grid_row: GridPlacement::start(row),
            grid_column: GridPlacement::start(col),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.10, 0.10, 0.18)),
        BorderColor::all(Color::srgb(0.24, 0.24, 0.44)),
        Outline { width: Val::Px(3.0), offset: Val::ZERO, color: Color::NONE },
        GlobalZIndex::default(),
        // should_block_lower: false lets drag events reach the slot underneath
        Pickable { should_block_lower: false, is_hoverable: true },
    ));

    if item.is_some() {
        slot.insert(InventoryItem);
    }

    // Hover → show tooltip with item name
    slot.observe(move |ev: On<Pointer<Over>>,
                       items: Query<&InventoryItem>,
                       mut vis_q: Query<&mut Visibility, With<TooltipRoot>>,
                       mut txt_q: Query<&mut Text, With<TooltipText>>| {
        if items.get(ev.event_target()).is_ok() {
            if let Ok(mut v) = vis_q.single_mut() { *v = Visibility::Visible; }
            if let Ok(mut t) = txt_q.single_mut() {
                **t = item_name.unwrap_or("").to_string();
            }
        }
    });
    slot.observe(|_: On<Pointer<Out>>,
                  mut vis_q: Query<&mut Visibility, With<TooltipRoot>>| {
        if let Ok(mut v) = vis_q.single_mut() { *v = Visibility::Hidden; }
    });

    // Drag start → white outline + elevate above other slots
    slot.observe(|ev: On<Pointer<DragStart>>,
                  mut q: Query<(&mut Outline, &mut GlobalZIndex)>| {
        if let Ok((mut outline, mut zidx)) = q.get_mut(ev.event_target()) {
            outline.color = Color::WHITE;
            zidx.0 = 10;
        }
    });

    // Drag → offset visual position by drag distance (leaves boundary freely)
    slot.observe(|ev: On<Pointer<Drag>>, mut q: Query<&mut UiTransform>| {
        if let Ok(mut t) = q.get_mut(ev.event_target()) {
            t.translation = Val2::px(ev.distance.x, ev.distance.y);
        }
    });

    // Drag end → reset visual offset and styling
    slot.observe(|ev: On<Pointer<DragEnd>>,
                  mut q: Query<(&mut UiTransform, &mut Outline, &mut GlobalZIndex)>| {
        if let Ok((mut t, mut outline, mut zidx)) = q.get_mut(ev.event_target()) {
            t.translation = Val2::ZERO;
            outline.color = Color::NONE;
            zidx.0 = 0;
        }
    });

    // Drop → swap grid positions so items visually exchange slots
    slot.observe(|ev: On<Pointer<DragDrop>>, mut q: Query<&mut Node>| {
        if let Ok([mut a, mut b]) = q.get_many_mut([ev.event_target(), ev.dropped]) {
            core::mem::swap(&mut a.grid_row, &mut b.grid_row);
            core::mem::swap(&mut a.grid_column, &mut b.grid_column);
        }
    });

    // Item visuals: colored icon + stack count in the corner
    if let Some((_, count, color)) = item {
        slot.with_children(|ch| {
            ch.spawn((
                Node {
                    width: Val::Px(SLOT_SIZE - 16.0),
                    height: Val::Px(SLOT_SIZE - 16.0),
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::FlexEnd,
                    padding: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BackgroundColor(color),
                Pickable::IGNORE,
            )).with_child((
                Text::new(if count > 1 { count.to_string() } else { String::new() }),
                TextFont::default().with_font_size(14.0),
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
    }

    slot.id()
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Keep the tooltip anchored to the cursor position.
fn position_tooltip(
    windows: Query<&Window>,
    mut q: Query<&mut Node, With<TooltipRoot>>,
) {
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    if let Ok(mut node) = q.single_mut() {
        node.left = Val::Px(cursor.x + 14.0);
        node.top = Val::Px(cursor.y + 14.0);
    }
}
