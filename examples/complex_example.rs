//! Complex UI example for `lava_ui_builder`.
//!
//! Demonstrates, on the scene (BSN) API:
//! - `scenes::collapsible` sections with toggle buttons
//! - `scenes::scrollable_list` with mouse wheel support
//! - Trade card display with grouped rows, built from domain data
//! - Game state and player activity panels
//! - `scenes::replace_children` for rebuilding a list in place at runtime
//!
//! The previous version carried its own copies of three systems that already live in
//! the library -- scroll handling, collapse toggling and collapsible visibility. They
//! are now just `LavaUiPlugin`.
//!
//! Run with: `cargo run --example complex_example`

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::{scenes, LavaTheme, LavaUiPlugin};

// ============================================================================
// Mock domain types (stand-ins for the original game project)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Egypt,
    Crete,
    Africa,
    Asia,
    Assyria,
    Babylon,
    Illyria,
}

impl std::fmt::Display for Faction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

const fn faction_color(faction: Faction) -> Color {
    match faction {
        Faction::Egypt => Color::srgb(0.9, 0.8, 0.3),
        Faction::Crete => Color::srgb(0.3, 0.6, 0.9),
        Faction::Africa => Color::srgb(0.6, 0.4, 0.2),
        Faction::Asia => Color::srgb(0.9, 0.5, 0.2),
        Faction::Assyria => Color::srgb(0.7, 0.2, 0.2),
        Faction::Babylon => Color::srgb(0.5, 0.3, 0.7),
        Faction::Illyria => Color::srgb(0.3, 0.7, 0.4),
    }
}

#[derive(Component)]
pub struct Player {
    pub faction: Faction,
    pub is_human: bool,
}

#[derive(Clone, Debug)]
pub struct CardStack {
    pub name: String,
    pub pile_value: usize,
    pub count: usize,
    pub suite_value: usize,
    pub is_commodity: bool,
    pub is_tradeable: bool,
}

#[derive(Component, Default)]
pub struct PlayerCards {
    pub stacks: Vec<CardStack>,
}

// ============================================================================
// App entry point
// ============================================================================

fn main() {
    App::new()
        // LavaUiPlugin brings the scroll, collapse and palette systems this example
        // used to carry its own copies of.
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .init_resource::<PlayerActivityLog>()
        .add_systems(
            Startup,
            (setup_camera, spawn_mock_players, setup_trade_ui).chain(),
        )
        .add_systems(
            Update,
            rebuild_activity_list.run_if(resource_changed::<PlayerActivityLog>),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// Spawn a handful of mock players so the UI has data to display.
fn spawn_mock_players(mut commands: Commands) {
    let players = [
        ("Ramesses", Faction::Egypt, true),
        ("Minos", Faction::Crete, false),
        ("Hannibal", Faction::Africa, false),
        ("Cyrus", Faction::Asia, false),
        ("Sargon", Faction::Assyria, false),
        ("Hammurabi", Faction::Babylon, false),
        ("Leonidas", Faction::Illyria, false),
    ];

    for (ruler, faction, is_human) in players {
        let mut stacks = Vec::new();
        // Give each player some mock trade cards
        for pile in 1..=5 {
            stacks.push(CardStack {
                name: format!("Commodity {pile}"),
                pile_value: pile,
                count: pile.min(3),
                suite_value: pile.saturating_mul(pile).saturating_mul(pile.min(3)),
                is_commodity: true,
                is_tradeable: true,
            });
        }
        if is_human {
            // Human player gets a calamity card too
            stacks.push(CardStack {
                name: "Famine".to_string(),
                pile_value: 3,
                count: 1,
                suite_value: 0,
                is_commodity: false,
                is_tradeable: false,
            });
        }

        commands.spawn((
            Player { faction, is_human },
            Name::new(format!("{ruler} of {faction:?}")),
            PlayerCards { stacks },
        ));
    }
}

// ============================================================================
// UI setup
// ============================================================================

#[allow(clippy::too_many_lines)]
// ============================================================================
// UI marker components
// ============================================================================
//
// Components used in `bsn!` need `Default + Clone`.

#[derive(Component, Default, Clone)]
pub struct TradeCardUiRoot;

#[derive(Component, Default, Clone)]
pub struct TradeCardList;

#[derive(Component, Default, Clone)]
pub struct GameStateDisplay;

#[derive(Component, Default, Clone)]
pub struct PlayerActivityListContainer;

// ============================================================================
// Resources
// ============================================================================

#[derive(Resource, Default)]
pub struct PlayerActivityLog {
    pub activities: HashMap<Entity, String>,
}

impl PlayerActivityLog {
    pub fn get(&self, player: Entity) -> &str {
        self.activities
            .get(&player)
            .map_or("Waiting...", std::string::String::as_str)
    }
}

// ============================================================================
// UI
// ============================================================================

const PANEL_BG: Color = Color::srgba(0.1, 0.1, 0.1, 0.7);
const HEADING: Color = Color::srgb(1.0, 0.8, 0.0);

/// A scene-spawning system: the whole tree is derived from world data, so it needs the
/// queries. One `spawn_scene` replaces the old builder walk.
fn setup_trade_ui(
    mut commands: Commands,
    players: Query<(&Name, &Player, &PlayerCards)>,
    activity_log: Res<PlayerActivityLog>,
) {
    let human_cards: Vec<CardStack> = players
        .iter()
        .find(|(_, player, _)| player.is_human)
        .map(|(_, _, cards)| cards.stacks.clone())
        .unwrap_or_default();

    let roster: Vec<(String, Faction, bool)> = players
        .iter()
        .map(|(name, player, _)| (name.to_string(), player.faction, player.is_human))
        .collect();

    let activity = activity_log.get(Entity::PLACEHOLDER).to_string();

    commands.spawn_scene(bsn! {
        TradeCardUiRoot
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Row,
            padding: {UiRect::all(px(8.0))},
            column_gap: {px(8.0)},
        }
        Children [
            trade_card_panel(&human_cards),
            game_info_panel(&roster, &activity),
        ]
    });
}

/// Left side: the human player's cards, grouped by pile value, in a scrollable box.
fn trade_card_panel(stacks: &[CardStack]) -> impl Scene {
    let rows: Vec<_> = (1..=9)
        .filter_map(|pile_value| {
            let mut pile: Vec<&CardStack> =
                stacks.iter().filter(|s| s.pile_value == pile_value).collect();
            if pile.is_empty() {
                return None;
            }
            // Commodities first, then the rest.
            pile.sort_by_key(|s| i32::from(!s.is_commodity));
            Some(trade_card_row(pile_value, &pile))
        })
        .collect();

    scenes::collapsible(
        "Trade Cards",
        false,
        bsn_list![(
            TradeCardList
            scenes::scrollable_list_bounded(0.0, 500.0)
            Node { width: {px(340.0)}, padding: {UiRect::all(px(4.0))} }
            BackgroundColor(PANEL_BG)
            Children [ {rows} ]
        )],
    )
}

fn trade_card_row(pile_value: usize, stacks: &[&CardStack]) -> impl Scene {
    let cards: Vec<_> = stacks.iter().map(|stack| trade_card(stack)).collect();
    bsn! {
        scenes::row(0.0)
        Node {
            width: percent(100),
            height: {px(70.0)},
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
        }
        Children [
            scenes::text(format!("{pile_value}:"), 20.0, Color::WHITE),
            {cards},
        ]
    }
}

fn trade_card(stack: &CardStack) -> impl Scene {
    let name = stack.name.clone();
    let subtitle = if stack.is_commodity {
        format!("x{} = {}", stack.count, stack.suite_value)
    } else if stack.is_tradeable {
        "Tradeable".to_string()
    } else {
        "Non-Tradeable".to_string()
    };
    bsn! {
        scenes::column(0.0)
        Node {
            width: {px(120.0)},
            height: {px(60.0)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(2.0))},
            margin: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(4.0))},
        }
        BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.8))
        Children [
            scenes::text(name, 16.0, Color::WHITE),
            scenes::text(subtitle, 12.0, Color::srgb(0.8, 0.8, 0.8)),
        ]
    }
}

/// Right side: static game state plus the live activity list.
fn game_info_panel(roster: &[(String, Faction, bool)], activity: &str) -> impl Scene {
    let census: Vec<_> = roster
        .iter()
        .enumerate()
        .map(|(i, (name, faction, is_human))| {
            let tag = if *is_human { " (YOU)" } else { "" };
            let line = format!("{}. {name}{tag}", i.saturating_add(1));
            scenes::text(line, 14.0, faction_color(*faction))
        })
        .collect();

    let activity_rows: Vec<_> = roster
        .iter()
        .map(|(name, faction, is_human)| {
            activity_row(name, *faction, *is_human, activity)
        })
        .collect();

    scenes::collapsible(
        "Game Info",
        false,
        bsn_list![(
            scenes::column(0.0)
            Node { width: {px(500.0)}, padding: {UiRect::all(px(4.0))} }
            BackgroundColor(PANEL_BG)
            Children [
                scenes::text("Game State", 20.0, HEADING),
                (
                    GameStateDisplay
                    scenes::column(0.0)
                    Node {
                        width: percent(100),
                        padding: {UiRect::all(px(4.0))},
                        margin: {UiRect::bottom(px(8.0))},
                    }
                    Children [
                        scenes::label("State: Playing"),
                        scenes::label("Activity: Trade"),
                        scenes::label("Round: 3"),
                        scenes::text("Census Order:", 16.0, HEADING),
                        {census},
                    ]
                ),
                scenes::text("Player Activity", 20.0, HEADING),
                (
                    PlayerActivityListContainer
                    scenes::scrollable_list_bounded(0.0, 300.0)
                    Node { padding: {UiRect::all(px(4.0))} }
                    Children [ {activity_rows} ]
                ),
            ]
        )],
    )
}

fn activity_row(name: &str, faction: Faction, is_human: bool, activity: &str) -> impl Scene {
    let color = faction_color(faction);
    let display_name = if is_human {
        format!("{name} (YOU): ")
    } else {
        format!("{name}: ")
    };
    let activity = activity.to_string();
    bsn! {
        scenes::row(0.0)
        Node {
            width: percent(100),
            height: {px(50.0)},
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(4.0))},
            margin: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(4.0))},
        }
        BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.8))
        Children [
            // Faction colour badge.
            (
                Node {
                    width: {px(18.0)},
                    height: {px(18.0)},
                    margin: {UiRect::all(px(4.0))},
                    border_radius: {BorderRadius::MAX},
                }
                BackgroundColor(color)
            ),
            scenes::text(display_name, 14.0, color),
            scenes::text(activity, 14.0, Color::WHITE),
        ]
    }
}

// ============================================================================
// Runtime rebuild
// ============================================================================

/// Rebuild the activity list in place when the log changes.
///
/// `scenes::replace_children` despawns the rows and spawns fresh ones, but keeps the
/// container entity -- so the `PlayerActivityListContainer` marker, its `ScrollPosition`
/// and anything else holding that id all survive. This is the scene replacement for
/// `UIBuilder::start_from_entity(.., clear_children: true)`.
fn rebuild_activity_list(
    mut commands: Commands,
    log: Res<PlayerActivityLog>,
    players: Query<(&Name, &Player)>,
    container: Single<Entity, With<PlayerActivityListContainer>>,
) {
    let activity = log.get(Entity::PLACEHOLDER).to_string();
    let rows: Vec<_> = players
        .iter()
        .map(|(name, player)| {
            activity_row(name.as_str(), player.faction, player.is_human, &activity)
        })
        .collect();
    scenes::replace_children(&mut commands, *container, rows);
}
