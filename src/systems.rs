use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;

use crate::{CollapseToggleButton, Collapsible, CollapsibleContent, InteractionPalette, LavaTheme, ProgressBar, ProgressBarFill, WorldFollower};

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
    theme: Option<Res<LavaTheme>>,
) {
    let (bg_normal, bg_hovered, bg_pressed) = theme.as_ref().map(|t| {
        (t.button.collapsible_bg, t.button.collapsible_bg_hovered, t.button.collapsible_bg_pressed)
    }).unwrap_or_else(|| {
        let d = crate::ButtonTheme::default();
        (d.collapsible_bg, d.collapsible_bg_hovered, d.collapsible_bg_pressed)
    });

    for (interaction, toggle_btn, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(bg_pressed);
                if let Ok(mut collapsible) = collapsible_query.get_mut(toggle_btn.target) {
                    collapsible.collapsed = !collapsible.collapsed;
                }
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(bg_hovered);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(bg_normal);
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
        .map(|rect| rect.min)
        .unwrap_or(Vec2::ZERO);
    for (entity, follower, mut node) in followers.iter_mut() {
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

#[cfg(test)]
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

