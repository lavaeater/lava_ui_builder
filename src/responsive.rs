//! Responsive layout: change a node's layout when the window is narrower than a
//! breakpoint, and put it back when it widens again.
//!
//! ```ignore
//! // A row of [image | buttons] that becomes a column, image on top, on a phone:
//! @scenes::row(24.0) @scenes::stack_below(500.0)
//! // Buttons that fill the screen width when narrow:
//! @scenes::button("Play") @scenes::full_width_below(500.0)
//! // Anything else: list the fields to override.
//! Responsive { breakpoint: 500.0, narrow: NarrowLayout { padding: {Some(UiRect::all(px(8.0)))} } }
//! ```
//!
//! The breakpoint is in logical window pixels (what the OS calls the window's width), not
//! UI pixels, so it does not move when `UiScale` changes.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// `Node` fields to override while the window is narrower than [`Responsive::breakpoint`].
/// `None` leaves a field alone.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NarrowLayout {
    pub flex_direction: Option<FlexDirection>,
    pub width: Option<Val>,
    pub max_width: Option<Val>,
    pub min_width: Option<Val>,
    pub display: Option<Display>,
    pub padding: Option<UiRect>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub row_gap: Option<Val>,
    pub column_gap: Option<Val>,
}

/// Apply [`NarrowLayout`] below `breakpoint` window pixels; restore the node's own values
/// above it.
#[derive(Component, Clone, Debug, Default)]
pub struct Responsive {
    pub breakpoint: f32,
    pub narrow: NarrowLayout,
    /// The node as it was before the narrow layout went on, while it is on.
    saved: Option<Node>,
}

/// Turn a row into a column below `breakpoint` window pixels.
pub fn stack_below(breakpoint: f32) -> impl Scene {
    bsn! {
        Responsive {
            breakpoint: {breakpoint},
            narrow: NarrowLayout { flex_direction: {Some(FlexDirection::Column)} },
        }
    }
}

/// Make a node fill its parent's width below `breakpoint` window pixels.
pub fn full_width_below(breakpoint: f32) -> impl Scene {
    bsn! {
        Responsive {
            breakpoint: {breakpoint},
            narrow: NarrowLayout { width: {Some(percent(100))}, max_width: {Some(Val::Auto)} },
        }
    }
}

macro_rules! for_each_field {
    ($m:ident) => {
        $m!(flex_direction);
        $m!(width);
        $m!(max_width);
        $m!(min_width);
        $m!(display);
        $m!(padding);
        $m!(align_items);
        $m!(justify_content);
        $m!(row_gap);
        $m!(column_gap);
    };
}

pub(crate) fn apply_responsive(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut nodes: Query<(&mut Responsive, &mut Node)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let width = window.width();
    for (mut responsive, mut node) in &mut nodes {
        let narrow = width < responsive.breakpoint;
        match (narrow, responsive.saved.is_some()) {
            (true, false) => {
                responsive.saved = Some(node.clone());
                let overrides = &responsive.narrow;
                macro_rules! apply {
                    ($f:ident) => {
                        if let Some(value) = overrides.$f {
                            node.$f = value;
                        }
                    };
                }
                for_each_field!(apply);
            }
            (false, true) => {
                let Some(saved) = responsive.saved.take() else {
                    continue;
                };
                let overrides = &responsive.narrow;
                macro_rules! restore {
                    ($f:ident) => {
                        if overrides.$f.is_some() {
                            node.$f = saved.$f;
                        }
                    };
                }
                for_each_field!(restore);
            }
            _ => {}
        }
    }
}
