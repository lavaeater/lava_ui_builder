use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

use crate::{CollapseToggleButton, Collapsible, CollapsibleContent, InteractionPalette};

// ============================================================================
// Scroll handling
// ============================================================================

/// Generic mouse wheel scroll handler for any entity with `ScrollPosition`.
/// Applies scroll delta to hovered scrollable elements.
pub fn handle_scroll_input(
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

// ============================================================================
// Collapsible toggle & visibility
// ============================================================================

/// Handle clicking the collapse/expand toggle button.
pub fn handle_collapse_toggle(
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
pub fn update_collapsible_visibility(
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

// ============================================================================
// InteractionPalette system
// ============================================================================

/// Apply `InteractionPalette` colors based on `Interaction` state changes.
pub fn apply_interaction_palette(
    mut query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut bg) in &mut query {
        *bg = BackgroundColor(match *interaction {
            Interaction::Pressed => palette.pressed,
            Interaction::Hovered => palette.hovered,
            Interaction::None => palette.none,
        });
    }
}

// ============================================================================
// Spawn helper (non-builder)
// ============================================================================

/// Helper function to spawn a collapsible section with content using raw Commands.
/// Returns `(collapsible_entity, content_entity)`.
pub fn spawn_collapsible_section(
    commands: &mut Commands,
    label: &str,
    initially_collapsed: bool,
) -> (Entity, Entity) {
    let collapsible_entity = commands
        .spawn((
            if initially_collapsed {
                Collapsible::collapsed(label)
            } else {
                Collapsible::new(label)
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Auto,
                ..Default::default()
            },
        ))
        .id();

    commands.entity(collapsible_entity).with_children(|parent| {
        parent
            .spawn((
                Button,
                CollapseToggleButton {
                    target: collapsible_entity,
                },
                Node {
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                    margin: UiRect::bottom(Val::Px(4.0)),
                    ..Default::default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ))
            .with_child((
                Text::new(if initially_collapsed {
                    format!("▶ {}", label)
                } else {
                    format!("▼ {}", label)
                }),
                TextFont {
                    font_size: 12.0,
                    ..Default::default()
                },
                TextColor(Color::WHITE),
            ));
    });

    let content_entity = commands
        .spawn((
            CollapsibleContent {
                parent: collapsible_entity,
            },
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                display: if initially_collapsed {
                    Display::None
                } else {
                    Display::Flex
                },
                ..Default::default()
            },
        ))
        .id();

    commands
        .entity(collapsible_entity)
        .add_child(content_entity);

    (collapsible_entity, content_entity)
}
