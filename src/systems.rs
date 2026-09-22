use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::{HoverMap, Hovered};
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::Activate;

use crate::{
    CollapseToggleButton, Collapsible, CollapsibleContent, InteractionPalette, ProgressBar,
    ProgressBarFill, WorldFollower,
};

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

/// Flip a [`Collapsible`]'s state when its [`CollapseToggleButton`] is activated.
///
/// Registered as a global observer rather than a per-entity one so that toggle buttons
/// spawned by any route -- builder, bundle function or scene -- are handled without
/// having to remember to attach anything. `Activate` is emitted by
/// [`bevy::ui_widgets::Button`] on release and on ENTER/SPACE when focused, so the
/// toggle works with the keyboard for free.
///
/// The button's own colors are handled by [`InteractionPalette`], not here.
pub fn toggle_collapsible_on_activate(
    activate: On<Activate>,
    toggles: Query<&CollapseToggleButton>,
    mut collapsibles: Query<&mut Collapsible>,
) {
    let Ok(toggle) = toggles.get(activate.entity) else {
        return;
    };
    if let Ok(mut collapsible) = collapsibles.get_mut(toggle.target) {
        collapsible.collapsed = !collapsible.collapsed;
    }
}

/// Update visibility of collapsible content based on collapsed state.
pub fn update_collapsible_visibility(
    collapsible_query: Query<(Entity, &Collapsible), Changed<Collapsible>>,
    mut content_query: Query<(&CollapsibleContent, &mut Node)>,
    button_text_query: Query<(&CollapseToggleButton, &Children)>,
    mut text_query: Query<&mut Text>,
) {
    for (collapsible_entity, collapsible) in collapsible_query.iter() {
        for (content, mut node) in &mut content_query {
            if content.parent == collapsible_entity {
                node.display = if collapsible.collapsed {
                    Display::None
                } else {
                    Display::Flex
                };
            }
        }

        for (toggle_btn, children) in button_text_query {
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
// Progress bar sync
// ============================================================================

/// Update the fill node width whenever a [`ProgressBar`]'s `value` changes.
pub fn sync_progress_bars(
    bar_query: Query<(&ProgressBar, &Children), Changed<ProgressBar>>,
    mut fill_query: Query<&mut Node, With<ProgressBarFill>>,
) {
    for (bar, children) in &bar_query {
        for child in children.iter() {
            if let Ok(mut node) = fill_query.get_mut(child) {
                node.width = Val::Percent(bar.value.clamp(0.0, 1.0) * 100.0);
            }
        }
    }
}

// ============================================================================
// InteractionPalette system
// ============================================================================

// ============================================================================
// WorldFollower — position a UI node at a world-space entity's screen position
// ============================================================================

/// Move each UI node with [`WorldFollower`] to track its target entity's screen position.
/// Despawns the follower if the target no longer exists.
pub fn world_follower_system(
    mut followers: Query<(Entity, &WorldFollower, &mut Node)>,
    transforms: Query<&GlobalTransform>,
    mut commands: Commands,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    ui_scale: Res<UiScale>,
) {
    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };
    let origin = camera
        .logical_viewport_rect()
        .map_or(Vec2::ZERO, |rect| rect.min);
    for (entity, follower, mut node) in &mut followers {
        let Ok(tr) = transforms.get(follower.target) else {
            commands.entity(entity).despawn();
            continue;
        };
        if let Ok(pos) = camera.world_to_viewport(camera_transform, tr.translation()) {
            node.left = Val::Px(follower_axis(origin.x, pos.x, follower.offset.x, ui_scale.0));
            node.top = Val::Px(follower_axis(origin.y, pos.y, follower.offset.y, ui_scale.0));
        }
    }
}

/// Turn a viewport-relative screen coordinate into the `Val::Px` that puts a UI node
/// there. Two corrections, both invisible in the common case of a full-window camera at
/// scale 1:
///
/// - **Viewport origin.** `Camera::world_to_viewport` is relative to the camera's
///   viewport, but the node is laid out in window space. A camera clipped to part of the
///   window — a split-pane editor, a minimap — offsets every follower by the pane origin
///   unless it is added back.
/// - **`UiScale`.** It multiplies every `Val::Px`, so a node placed at a raw screen
///   coordinate renders at `scale` times that, dragged toward the top-left corner. It has
///   to be divided out. `offset` stays outside the division: it is authored in UI units
///   (it centres the node on its target) and is scaled along with the node's own size.
fn follower_axis(origin: f32, viewport_pos: f32, offset: f32, ui_scale: f32) -> f32 {
    let scale = if ui_scale > 0.0 { ui_scale } else { 1.0 };
    ((origin + viewport_pos) / scale + offset).round()
}

/// Apply [`InteractionPalette`] colors from the headless-widget interaction state.
///
/// Reads [`Hovered`] (maintained by the picking backend for every entity carrying the
/// component) and [`Pressed`] (added and removed by `bevy_ui_widgets`' button observers)
/// in preference to the legacy [`Interaction`] component: `bevy::ui_widgets::Button` does
/// not require `Interaction`, so a palette driven by it never fired on those entities.
///
/// [`Interaction`] is still honoured as a fallback, so a palette on an entity built with
/// the legacy `bevy_ui::widget::Button` keeps working.
///
/// The query is unfiltered on purpose. `Pressed` is a marker that is *removed* on
/// release, and removal is invisible to `Changed`/`Added` filters, so the state is
/// recomputed every frame instead. The write is guarded by an equality check, so change
/// detection on `BackgroundColor` still only fires when the color actually moves.
#[expect(
    clippy::type_complexity,
    reason = "a Bevy query tuple; splitting it into a type alias hurts more than it helps"
)]
pub fn apply_interaction_palette(
    mut query: Query<(
        &InteractionPalette,
        Option<&Hovered>,
        Has<Pressed>,
        Option<&Interaction>,
        &mut BackgroundColor,
    )>,
) {
    for (palette, hovered, pressed, interaction, mut bg) in &mut query {
        // `Hovered`/`Pressed` when present; otherwise fall back to the legacy
        // `Interaction`, so an entity built with `bevy_ui::widget::Button` (which requires
        // `Interaction` but not `Hovered`) still gets its colours.
        let target = if pressed || interaction == Some(&Interaction::Pressed) {
            palette.pressed
        } else if hovered.is_some_and(Hovered::get)
            || (hovered.is_none() && interaction == Some(&Interaction::Hovered))
        {
            palette.hovered
        } else {
            palette.none
        };
        if bg.0 != target {
            bg.0 = target;
        }
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // exact, representable f32 values in assert_eq!
mod tests {
    use super::follower_axis;

    #[test]
    fn a_full_window_camera_at_scale_one_is_just_the_offset() {
        assert_eq!(follower_axis(0.0, 500.0, -30.0, 1.0), 470.0);
    }

    /// The pane origin has to survive the scale division, or a follower in a split-pane
    /// screen lands short of its target by a fraction of the pane width.
    #[test]
    fn a_clipped_viewport_lands_on_the_target_not_the_window_corner() {
        // Pane starts 330px in; target 439px into the pane; UI drawn at 0.66.
        let left = follower_axis(330.0, 439.0, 0.0, 0.66);
        assert_eq!((left * 0.66).round(), 769.0, "renders back onto the target");
    }

    #[test]
    fn ui_scale_is_divided_out_so_the_node_renders_where_asked() {
        let left = follower_axis(0.0, 900.0, 0.0, 0.5);
        assert_eq!(left, 1800.0);
        assert_eq!(left * 0.5, 900.0, "round-trips through the scale");
    }

    #[test]
    fn a_degenerate_scale_falls_back_to_one_instead_of_diverging() {
        assert_eq!(follower_axis(0.0, 500.0, 0.0, 0.0), 500.0);
    }
}
