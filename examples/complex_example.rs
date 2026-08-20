//! Complex UI example for `lava_ui_builder`.
//!
//! Demonstrates:
//! - Collapsible sections with toggle buttons
//! - Scrollable containers with mouse wheel support
//! - Trade card display with grouped rows
//! - Game state and player activity panels
//! - Dynamic UI rebuilding with `start_from_entity`
//!
//! Run with: `cargo run --example complex_example`

use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use lava_ui_builder::{
    CollapseToggleButton, Collapsible, CollapsibleContent, LavaTheme, TextStyle, UIBuilder,
};

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
        write!(f, "{:?}", self)
    }
}

fn faction_color(faction: Faction) -> Color {
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
// UI marker components
// ============================================================================

#[derive(Component, Default)]
pub struct TradeCardUiRoot;

#[derive(Component, Default)]
pub struct TradeCardList;

#[derive(Component, Default)]
pub struct GameStateDisplay;

#[derive(Component, Default)]
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
            .map(|s| s.as_str())
            .unwrap_or("Waiting...")
    }
}

// ============================================================================
// App entry point
// ============================================================================

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .init_resource::<PlayerActivityLog>()
        .add_systems(
            Startup,
            (setup_camera, spawn_mock_players, setup_trade_ui).chain(),
        )
        .add_systems(
            Update,
            (
                handle_scroll_input,
                handle_collapse_toggle,
                update_collapsible_visibility,
            ),
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
                name: format!("Commodity {}", pile),
                pile_value: pile,
                count: pile.min(3),
                suite_value: pile * pile * pile.min(3),
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
            Name::new(format!("{} of {:?}", ruler, faction)),
            PlayerCards { stacks },
        ));
    }
}

// ============================================================================
// UI setup
// ============================================================================

fn setup_trade_ui(
    commands: Commands,
    theme: Res<LavaTheme>,
    players: Query<(&Name, &Player, &PlayerCards)>,
    activity_log: Res<PlayerActivityLog>,
) {
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.component::<TradeCardUiRoot>().set_node(Node {
        width: percent(100.0),
        height: percent(100.0),
        flex_direction: FlexDirection::Row,
        padding: UiRect::all(Val::Px(8.0)),
        column_gap: Val::Px(8.0),
        ..default()
    });

    // ── Left side: Collapsible Trade Cards ────────────────────────────
    ui.with_collapsible("Trade Cards", false, |cards_section| {
        cards_section
            .component::<TradeCardList>()
            .width_px(340.0)
            .height_px(500.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.7))
            .with_overflow(Overflow::scroll_y())
            .insert(ScrollPosition::default())
            .padding_all_px(4.0);

        // Find the human player's cards
        for (_name, player, cards) in players.iter() {
            if player.is_human {
                build_trade_card_list(cards_section, &cards.stacks);
            }
        }
    });

    // ── Right side: Collapsible Game Info ──────────────────────────────
    ui.with_collapsible("Game Info", false, |info| {
        info.width_px(500.0)
            .bg_color(Color::srgba(0.1, 0.1, 0.1, 0.7))
            .padding_all_px(4.0);

        // Game State sub-section
        info.add_text_child(
            "Game State",
            Some(TextStyle::size_color(20.0, Color::srgb(1.0, 0.8, 0.0))),
        );
        info.with_child(|state| {
            state
                .component::<GameStateDisplay>()
                .width_percent(100.0)
                .display_flex()
                .flex_column()
                .padding_all_px(4.0)
                .margin(UiRect::bottom(Val::Px(8.0)));

            state.add_text_child("State: Playing", None);
            state.add_text_child("Activity: Trade", None);
            state.add_text_child("Round: 3", None);

            // Census order
            state.add_text_child(
                "Census Order:",
                Some(TextStyle::size_color(16.0, Color::srgb(1.0, 0.8, 0.0))),
            );
            for (i, (name, player, _cards)) in players.iter().enumerate() {
                let color = faction_color(player.faction);
                let human_tag = if player.is_human { " (YOU)" } else { "" };
                state.add_text_child(
                    format!("{}. {}{}", i + 1, name, human_tag),
                    Some(TextStyle::size_color(14.0, color)),
                );
            }
        });

        // Player Activity sub-section
        info.add_text_child(
            "Player Activity",
            Some(TextStyle::size_color(20.0, Color::srgb(1.0, 0.8, 0.0))),
        );
        info.with_child(|list| {
            list.component::<PlayerActivityListContainer>()
                .width_percent(100.0)
                .height_px(300.0)
                .display_flex()
                .flex_column()
                .padding_all_px(4.0)
                .with_overflow(Overflow::scroll_y())
                .insert(ScrollPosition::default());

            // Populate with current activity data
            for (name, player, _cards) in players.iter() {
                let color = faction_color(player.faction);
                let display_name = if player.is_human {
                    format!("{} (YOU)", name)
                } else {
                    name.to_string()
                };

                list.with_child(|row| {
                    row.width_percent(100.0)
                        .height_px(50.0)
                        .display_flex()
                        .flex_row()
                        .align_items_center()
                        .padding_all_px(4.0)
                        .margin_all_px(2.0)
                        .bg_color(Color::srgba(0.15, 0.15, 0.2, 0.8))
                        .border_radius_all_px(4.0);

                    // Faction color badge
                    row.with_child(|badge| {
                        badge
                            .width_px(18.0)
                            .height_px(18.0)
                            .bg_color(color)
                            .border_radius_all_px(9.0)
                            .margin_all_px(4.0);
                    });

                    row.add_text_child(
                        format!("{}: ", display_name),
                        Some(TextStyle::size_color(14.0, color)),
                    );
                    row.add_text_child(
                        activity_log.get(Entity::PLACEHOLDER),
                        Some(TextStyle::size(14.0)),
                    );
                });
            }
        });
    });

    ui.build();
}

// ============================================================================
// Trade card rendering helpers
// ============================================================================

fn build_trade_card(ui: &mut UIBuilder, stack: &CardStack) {
    let small_font = 12.0;
    let medium_font = 16.0;

    ui.with_child(|card| {
        card.width_px(120.0)
            .height_px(60.0)
            .display_flex()
            .flex_column()
            .justify_center()
            .align_items_center()
            .padding_all_px(2.0)
            .margin_all_px(2.0)
            .bg_color(Color::srgba(0.2, 0.2, 0.3, 0.8))
            .border_radius_all_px(4.0);

        if stack.is_commodity {
            card.add_text_child(&stack.name, Some(TextStyle::size(medium_font)));
            card.add_text_child(
                format!("x{} = {}", stack.count, stack.suite_value),
                Some(TextStyle::size(small_font)),
            );
        } else {
            card.add_text_child(&stack.name, Some(TextStyle::size(medium_font)));
            card.add_text_child(
                if stack.is_tradeable {
                    "Tradeable"
                } else {
                    "Non-Tradeable"
                },
                Some(TextStyle::size(small_font)),
            );
        }
    });
}

fn build_trade_card_list(ui: &mut UIBuilder, stacks: &[CardStack]) {
    for pile_value in 1..=9 {
        let pile_stacks: Vec<_> = stacks
            .iter()
            .filter(|s| s.pile_value == pile_value)
            .collect();

        if pile_stacks.is_empty() {
            continue;
        }

        let mut sorted = pile_stacks;
        sorted.sort_by_key(|s| if s.is_commodity { 0 } else { 1 });

        ui.add_row(|row| {
            row.width_percent(100.0)
                .height_px(70.0)
                .justify_start()
                .align_items_center()
                .with_flex_shrink(0.0);

            row.add_text_child(format!("{}:", pile_value), Some(TextStyle::size(20.0)));

            for stack in &sorted {
                build_trade_card(row, stack);
            }
        });
    }
}

// ============================================================================
// Runtime systems
// ============================================================================

/// Handle mouse wheel scroll input for scrollable containers.
fn handle_scroll_input(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scroll_query: Query<&mut ScrollPosition>,
) {
    for mouse_wheel in mouse_wheel_events.read() {
        let dy = match mouse_wheel.unit {
            MouseScrollUnit::Line => mouse_wheel.y * 20.0,
            MouseScrollUnit::Pixel => mouse_wheel.y,
        };

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys() {
                if let Ok(mut scroll_position) = scroll_query.get_mut(*entity) {
                    scroll_position.y -= dy;
                    scroll_position.y = scroll_position.y.max(0.0);
                }
            }
        }
    }
}

/// Handle clicking the collapse/expand toggle button.
fn handle_collapse_toggle(
    mut interaction_query: Query<
        (&Interaction, &CollapseToggleButton, &mut BackgroundColor),
        Changed<Interaction>,
    >,
    mut collapsible_query: Query<&mut Collapsible>,
) {
    for (interaction, toggle_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.6, 0.4));
                if let Ok(mut collapsible) = collapsible_query.get_mut(toggle_btn.target) {
                    collapsible.collapsed = !collapsible.collapsed;
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.35, 0.35, 0.4));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
            }
        }
    }
}

/// Update visibility of collapsible content based on collapsed state.
fn update_collapsible_visibility(
    collapsible_query: Query<(Entity, &Collapsible), Changed<Collapsible>>,
    mut content_query: Query<(&CollapsibleContent, &mut Node)>,
    mut button_text_query: Query<(&CollapseToggleButton, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (collapsible_entity, collapsible) in collapsible_query.iter() {
        for (content, mut node) in content_query.iter_mut() {
            if content.parent == collapsible_entity {
                node.display = if collapsible.collapsed {
                    Display::None
                } else {
                    Display::Flex
                };
            }
        }

        for (toggle_btn, children) in button_text_query.iter_mut() {
            if toggle_btn.target == collapsible_entity {
                for child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        **text = if collapsible.collapsed {
                            format!("▶ {}", collapsible.label)
                        } else {
                            format!("▼ {}", collapsible.label)
                        };
                    }
                }
            }
        }
    }
}
