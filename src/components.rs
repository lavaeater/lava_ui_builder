//! Components shared by all three APIs (scene, bundle and builder).

use bevy::ecs::template::FromTemplate;
use bevy::prelude::*;

/// Interaction palette for hover/press color changes.
///
/// `apply_interaction_palette` turns it into the entity's `BackgroundColor` for the current
/// `Hovered` / `Pressed` state. Scene widgets do not set it directly: a
/// [`crate::Variant`] or [`crate::ThemedPalette`] fills it from the theme.
///
/// `Default` is required so the component can be patched field-by-field in `bsn!`.
#[derive(Component, Clone, Debug, Default)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

// ============================================================================
// Collapsible widget components
// ============================================================================

/// Marker component for a collapsible UI section.
/// When collapsed, the content is hidden and only the toggle button is shown.
#[derive(Component, Clone, Debug, Default)]
pub struct Collapsible {
    pub collapsed: bool,
    pub label: String,
}

impl Collapsible {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            collapsed: false,
            label: label.into(),
        }
    }

    pub fn collapsed(label: impl Into<String>) -> Self {
        Self {
            collapsed: true,
            label: label.into(),
        }
    }
}

/// Marker for the toggle button that collapses/expands a Collapsible section.
///
/// Derives [`FromTemplate`] rather than `Default` + `Clone` because of the [`Entity`]
/// field: that is what lets `bsn!` resolve a `#Name` reference into the real entity at
/// spawn time (`CollapseToggleButton { target: #Section }`).
#[derive(Component, Debug, FromTemplate)]
pub struct CollapseToggleButton {
    pub target: Entity,
}

/// Marker for the content container inside a Collapsible section.
///
/// See [`CollapseToggleButton`] for why this derives [`FromTemplate`].
#[derive(Component, Debug, FromTemplate)]
pub struct CollapsibleContent {
    pub parent: Entity,
}

/// Where `UIBuilder::with_collapsible`'s toggle button renders relative to its
/// content -- see `UIBuilder::with_collapsible_toggle_at_bottom`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CollapsibleTogglePosition {
    Top,
    Bottom,
}

// ============================================================================
// Progress bar / meter
// ============================================================================

/// Drives the fill of a progress bar or meter. Change `value` (0.0–1.0) and the
/// `sync_progress_bars` system resizes the inner [`ProgressBarFill`] node.
///
/// To drive it from game state without writing a system, bind it:
/// [`crate::BindMeter`].
#[derive(Component, Clone, Debug, Default)]
pub struct ProgressBar {
    /// Fill fraction, clamped to `0.0..=1.0`.
    pub value: f32,
    /// Only read by the bundle-API `progress_bar`; themed meters take the colour from a
    /// token on the fill instead.
    pub fill_color: Color,
}

/// Marker placed on the inner fill node of a progress bar / meter.
#[derive(Component, Clone, Debug, Default)]
pub struct ProgressBarFill;

// ============================================================================
// WorldFollower — UI node that tracks a world-space entity's screen position
// ============================================================================

/// Attach to a UI node to make it follow `target` in screen space. The follower system
/// makes the node `PositionType::Absolute` itself. `offset` is applied in UI pixels
/// relative to the projected position, so `-width / 2` centres a node over its target.
/// The follower despawns when its target does.
#[derive(Component, Debug, FromTemplate)]
pub struct WorldFollower {
    pub target: Entity,
    pub offset: Vec2,
}
