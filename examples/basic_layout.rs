//! Basic layout example for `lava_ui_builder`.
//!
//! Demonstrates:
//! - Creating a root UI node with `ui_root`
//! - Header and label text using `TextTheme`
//! - Imperative `UIBuilder` with rows, columns, panels, grids, and styling
//!
//! Run with: `cargo run --example basic_layout`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, (setup_camera, setup_ui))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_ui(commands: Commands, theme: Res<LavaTheme>) {
    // Both APIs share a single root node so they don't overlap.
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.set_node(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        ..default()
    });

    // ── Bundle-function approach ──────────────────────────────────────
    // `header` and `label` return `impl Bundle`; insert them on child nodes.
    ui.with_child(|c| { c.insert_bundle(header("Lava UI Builder", &theme.text)); });
    ui.with_child(|c| { c.insert_bundle(label("Bundle-function widgets compose with children![]", &theme.text)); });

    // ── Imperative UIBuilder approach ─────────────────────────────────
    // A simple column with text nodes
    ui.add_column(|col| {
        col.gap_px(8.0).align_items_center();
        col.text_node("── Imperative UIBuilder ──");
        col.text_node("Rows, columns, panels, and grids below");
    });

    // A row with colored boxes
    ui.add_row(|row| {
        row.gap_px(12.0).align_items_center();

        row.with_child(|box1| {
            box1.size_px(80.0, 80.0)
                .bg_color(Color::srgb(0.8, 0.2, 0.2))
                .border_radius_all_px(8.0);
        });
        row.with_child(|box2| {
            box2.size_px(80.0, 80.0)
                .bg_color(Color::srgb(0.2, 0.8, 0.2))
                .border_radius_all_px(8.0);
        });
        row.with_child(|box3| {
            box3.size_px(80.0, 80.0)
                .bg_color(Color::srgb(0.2, 0.2, 0.8))
                .border_radius_all_px(8.0);
        });
    });

    // A panel with a border
    ui.add_panel(|panel| {
        panel
            .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.9))
            .border(UiRect::all(Val::Px(2.0)), Color::srgb(0.4, 0.4, 0.6))
            .border_radius_all_px(12.0)
            .gap_px(6.0);

        panel.text_node("This is a panel");
        panel.text_node("It has padding, a background, and a border");
    });

    // A 3-column grid built from data
    let items: Vec<&str> = vec!["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"];

    ui.add_grid(3, |grid| {
        grid.width_px(400.0);
        grid.foreach_child(items, |cell, item| {
            cell.bg_color(Color::srgba(0.3, 0.3, 0.4, 0.8))
                .padding_all_px(8.0)
                .border_radius_all_px(4.0)
                .default_text(item);
        });
    });

    ui.build();
}
