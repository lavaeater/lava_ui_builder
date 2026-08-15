//! Scoreboard example for `lava_ui_builder` — Challenge 8.
//!
//! Demonstrates a grid layout driven by ECS component data.
//! Players are sorted by kills; rows alternate background colors.
//!
//! Run with: `cargo run --example scoreboard`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, (setup_camera, spawn_players, setup_ui).chain())
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── ECS data ─────────────────────────────────────────────────────────────────

#[derive(Component)]
struct PlayerStats {
    name: String,
    kills: u32,
    deaths: u32,
    assists: u32,
    team: Team,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Team {
    Alpha,
    Bravo,
}

fn spawn_players(mut commands: Commands) {
    let data = [
        ("Ramses",   14, 2,  5,  Team::Alpha),
        ("Leonidas", 12, 4,  8,  Team::Alpha),
        ("Caesar",   9,  6,  11, Team::Alpha),
        ("Hannibal", 17, 1,  3,  Team::Bravo),
        ("Cyrus",    7,  8,  14, Team::Bravo),
        ("Sargon",   4,  10, 6,  Team::Bravo),
    ];
    for (name, k, d, a, team) in data {
        commands.spawn(PlayerStats { name: name.into(), kills: k, deaths: d, assists: a, team });
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

fn setup_ui(commands: Commands, theme: Res<LavaTheme>, players: Query<&PlayerStats>) {
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

    ui.add_text_child("SCOREBOARD", Some(TextStyle::size_color(40.0, Color::WHITE)));

    // Sort players by kills descending
    let mut sorted: Vec<_> = players.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.kills));

    ui.with_child(|table| {
        table
            .display_flex().flex_column().gap_px(4.0)
            .padding_all_px(16.0)
            .bg_color(Color::srgba(0.05, 0.05, 0.12, 0.96))
            .border_all_px(2.0, Color::srgb(0.3, 0.3, 0.55))
            .border_radius_all_px(10.0);

        // Header
        table.with_child(|row| {
            header_row(row);
        });

        // Data rows
        for (i, stats) in sorted.iter().enumerate() {
            let bg = if i % 2 == 0 {
                Color::srgba(0.10, 0.10, 0.18, 0.9)
            } else {
                Color::srgba(0.14, 0.14, 0.22, 0.9)
            };
            let team_color = if stats.team == Team::Alpha {
                Color::srgb(0.3, 0.6, 1.0)
            } else {
                Color::srgb(1.0, 0.5, 0.3)
            };

            table.with_child(|row| {
                row.display_flex().flex_row()
                    .align_items_center()
                    .gap_px(8.0)
                    .padding_all_px(8.0)
                    .bg_color(bg)
                    .border_radius_all_px(4.0);

                // Team color dot
                row.with_child(|dot| {
                    dot.size_px(12.0, 12.0)
                        .bg_color(team_color)
                        .border_radius_all_px(6.0);
                });

                cell(row, &stats.name,          180.0, Color::WHITE);
                cell(row, &stats.kills.to_string(),   60.0, Color::srgb(0.6, 1.0, 0.6));
                cell(row, &stats.deaths.to_string(),  60.0, Color::srgb(1.0, 0.5, 0.5));
                cell(row, &stats.assists.to_string(), 60.0, Color::srgb(0.6, 0.8, 1.0));

                // KD ratio
                let kd = stats.kills as f32 / (stats.deaths as f32).max(1.0);
                cell(row, &format!("{kd:.2}"), 70.0, Color::srgb(1.0, 0.9, 0.4));
            });
        }
    });

    ui.build();
}

fn header_row(row: &mut UIBuilder) {
    row.display_flex().flex_row()
        .align_items_center()
        .gap_px(8.0)
        .padding_all_px(8.0)
        .bg_color(Color::srgb(0.18, 0.18, 0.32))
        .border_radius_all_px(4.0);

    // Spacer for the dot column
    row.with_child(|s| { s.size_px(12.0, 12.0); });

    for (label, width) in [
        ("Player",  180.0),
        ("K",        60.0),
        ("D",        60.0),
        ("A",        60.0),
        ("K/D",      70.0),
    ] {
        row.with_child(|cell| {
            cell.width_px(width)
                .with_text(label, Some(TextStyle::size_color(16.0, Color::srgb(1.0, 0.9, 0.4))));
        });
    }
}

fn cell(row: &mut UIBuilder, text: &str, width: f32, color: Color) {
    row.with_child(|c| {
        c.width_px(width)
            .with_text(text, Some(TextStyle::size_color(16.0, color)));
    });
}
