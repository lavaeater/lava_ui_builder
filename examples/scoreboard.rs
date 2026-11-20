//! Challenge 8: a scoreboard.
//!
//! - A grid with players as rows and avatar, name, K, D, A as columns.
//! - Avatars are images. The grid sizes itself to the content: `scenes::table` gives every
//!   column the width of its widest cell, so a long name widens the name column.
//! - The data lives in the ECS, one `Player` entity each. When any player changes, the
//!   table's rows are rebuilt from a query with `replace_children`.
//!
//! Press K to have someone score; Tab hides and shows the board.
//!
//! Run with: `cargo run --example scoreboard`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

#[derive(Component)]
struct Player {
    name: &'static str,
    avatar: usize,
    kills: u32,
    deaths: u32,
    assists: u32,
}

/// The table whose rows get rebuilt.
#[derive(Component, Default, Clone)]
struct Board;

#[derive(Resource)]
struct Shown(bool);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .insert_resource(Shown(true))
        .bind_resource::<Shown>()
        .add_systems(Startup, (players, ui.spawn()))
        .add_systems(Update, (simulate, rebuild_rows))
        .run();
}

fn ui() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @scenes::ui_root()
        Children [
            @scenes::card() @scenes::bind_visible(|s: &Shown| s.0)
            Children [
                @scenes::header("Scoreboard")
                --
                @scenes::table(5, 24.0, 8.0) Board
                --
                @scenes::caption("K: someone scores   Tab: hide / show")
            ]
        ]
    }
}

/// Header cells, then one row of five cells per player, best first.
fn rebuild_rows(
    changed: Query<(), Changed<Player>>,
    players: Query<&Player>,
    board: Query<(Entity, Ref<Board>)>,
    mut commands: Commands,
) {
    // The board spawns from a scene, so it may appear a frame after the players do.
    let Ok((board, marker)) = board.single() else {
        return;
    };
    if changed.is_empty() && !marker.is_added() {
        return;
    }
    let mut sorted: Vec<&Player> = players.iter().collect();
    sorted.sort_by_key(|p| std::cmp::Reverse(p.kills * 2 + p.assists));
    let header: Vec<_> = ["", "Player", "K", "D", "A"]
        .into_iter()
        .map(|h| bsn! { @scenes::caption(h) })
        .collect();
    let rows: Vec<_> = sorted
        .into_iter()
        .flat_map(|p| {
            [
                cell(Some(format!("avatars/{}.png", p.avatar)), String::new()),
                cell(None, p.name.to_string()),
                cell(None, p.kills.to_string()),
                cell(None, p.deaths.to_string()),
                cell(None, p.assists.to_string()),
            ]
        })
        .collect();
    scenes::replace_children(&mut commands, board, (header, rows));
}

/// One table cell: an avatar or a label. A single type for both, so a row is a `Vec`.
fn cell(avatar: Option<String>, text: String) -> impl Scene {
    let label = avatar.is_none().then(|| bsn! { @scenes::label(text) });
    (avatar.map(|path| bsn! { @scenes::icon(path, 40.0) }), label)
}

fn players(mut commands: Commands) {
    let roster = [
        ("Ramses", 14, 2, 5),
        ("Leonidas the Unyielding", 12, 4, 8),
        ("Caesar", 9, 6, 11),
        ("Hannibal", 17, 1, 3),
        ("Cyrus", 7, 8, 14),
        ("Sargon", 4, 10, 6),
    ];
    for (avatar, (name, kills, deaths, assists)) in roster.into_iter().enumerate() {
        commands.spawn(Player { name, avatar, kills, deaths, assists });
    }
}

fn simulate(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut players: Query<&mut Player>,
    mut shown: ResMut<Shown>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        shown.0 = !shown.0;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        // Pick a scorer and a victim from the clock: no rng crate needed for a demo.
        let n = players.iter().len().max(1);
        let tick = (time.elapsed_secs() * 1000.0) as usize;
        for (i, mut p) in players.iter_mut().enumerate() {
            if i == tick % n {
                p.kills += 1;
            } else if i == (tick / 7) % n {
                p.deaths += 1;
            }
        }
    }
}
