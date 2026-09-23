//! Scoreboard example for `lava_ui_builder` — Challenge 8, on the scene (BSN) API.
//!
//! Demonstrates the data-driven shape: an ordinary system reads ECS data, builds a
//! `Vec<impl Scene>` of rows, and spawns the whole table in one `commands.spawn_scene`.
//! This is what `UIBuilder::foreach_child` was for.
//!
//! Run with: `cargo run --example scoreboard`

use bevy::prelude::*;
use lava_ui_builder::{scenes, LavaTheme, LavaUiPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
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
    kills: i32,
    deaths: i32,
    assists: i32,
    team: Team,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Team {
    Alpha,
    Bravo,
}

impl Team {
    const fn color(self) -> Color {
        match self {
            Self::Alpha => Color::srgb(0.3, 0.6, 1.0),
            Self::Bravo => Color::srgb(1.0, 0.5, 0.3),
        }
    }
}

fn spawn_players(mut commands: Commands) {
    let data = [
        ("Ramses", 14, 2, 5, Team::Alpha),
        ("Leonidas", 12, 4, 8, Team::Alpha),
        ("Caesar", 9, 6, 11, Team::Alpha),
        ("Hannibal", 17, 1, 3, Team::Bravo),
        ("Cyrus", 7, 8, 14, Team::Bravo),
        ("Sargon", 4, 10, 6, Team::Bravo),
    ];
    for (name, k, d, a, team) in data {
        commands.spawn(PlayerStats {
            name: name.into(),
            kills: k,
            deaths: d,
            assists: a,
            team,
        });
    }
}

// ── UI ────────────────────────────────────────────────────────────────────────

const HEADER_COLOR: Color = Color::srgb(1.0, 0.9, 0.4);
const CELL_SIZE: f32 = 16.0;
const COLUMNS: [(&str, f32); 5] = [
    ("Player", 180.0),
    ("K", 60.0),
    ("D", 60.0),
    ("A", 60.0),
    ("K/D", 70.0),
];

/// A scene-spawning *system*, not a `scene.spawn()` one: the rows depend on world data,
/// so this needs the query. `commands.spawn_scene` is the only difference.
fn setup_ui(mut commands: Commands, players: Query<&PlayerStats>) {
    let mut sorted: Vec<&PlayerStats> = players.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.kills));

    let rows: Vec<_> = sorted
        .iter()
        .enumerate()
        .map(|(i, stats)| player_row(i, stats))
        .collect();

    commands.spawn_scene(bsn! {
        scenes::ui_root()
        Children [
            scenes::text("SCOREBOARD", 40.0, Color::WHITE),
            (
                scenes::column(4.0)
                Node {
                    padding: {UiRect::all(px(16.0))},
                    border: {UiRect::all(px(2.0))},
                    border_radius: {BorderRadius::all(px(10.0))},
                }
                BackgroundColor({Color::srgba(0.05, 0.05, 0.12, 0.96)})
                BorderColor::all(Color::srgb(0.3, 0.3, 0.55))
                Children [
                    header_row(),
                    {rows},
                ]
            ),
        ]
    });
}

fn table_row(bg: Color) -> impl Scene {
    bsn! {
        scenes::row(8.0)
        Node {
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(8.0))},
            border_radius: {BorderRadius::all(px(4.0))},
        }
        BackgroundColor({bg})
    }
}

fn cell(content: impl Into<String>, width: f32, color: Color) -> impl Scene {
    bsn! {
        scenes::text(content.into(), CELL_SIZE, color)
        Node { width: {px(width)} }
    }
}

/// A fixed-size box: the team-colour dot, or the blank that lines the header up with it.
fn dot(color: Option<Color>) -> impl Scene {
    let color = color.unwrap_or(Color::NONE);
    bsn! {
        Node { width: {px(12.0)}, height: {px(12.0)}, border_radius: {BorderRadius::MAX} }
        BackgroundColor({color})
    }
}

fn header_row() -> impl Scene {
    let cells: Vec<_> = COLUMNS
        .into_iter()
        .map(|(label, width)| cell(label, width, HEADER_COLOR))
        .collect();
    bsn! {
        table_row(Color::srgb(0.18, 0.18, 0.32))
        Children [
            dot(None),
            {cells},
        ]
    }
}

fn player_row(index: usize, stats: &PlayerStats) -> impl Scene {
    let bg = if index.is_multiple_of(2) {
        Color::srgba(0.10, 0.10, 0.18, 0.9)
    } else {
        Color::srgba(0.14, 0.14, 0.22, 0.9)
    };
    let kd = f64::from(stats.kills) / f64::from(stats.deaths).max(1.0);
    let cells = vec![
        cell(stats.name.clone(), 180.0, Color::WHITE),
        cell(stats.kills.to_string(), 60.0, Color::srgb(0.6, 1.0, 0.6)),
        cell(stats.deaths.to_string(), 60.0, Color::srgb(1.0, 0.5, 0.5)),
        cell(stats.assists.to_string(), 60.0, Color::srgb(0.6, 0.8, 1.0)),
        cell(format!("{kd:.2}"), 70.0, HEADER_COLOR),
    ];
    bsn! {
        table_row(bg)
        Children [
            dot(Some(stats.team.color())),
            {cells},
        ]
    }
}
