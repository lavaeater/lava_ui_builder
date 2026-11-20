//! Layout scenes: containers that set up a `Node` (and, for surfaces, a themed
//! background). The caller supplies `Children [ .. ]`.
//!
//! They exist because the flex/grid incantations are easy to get subtly wrong, not
//! because BSN needs them -- `scenes::row(8.0)` is exactly a flex-row `Node`.

use bevy::prelude::*;

use crate::{ColorToken, ThemedBackground, ThemedBorderColor};

/// A root UI node that fills the window and centers its content in a column.
///
/// The scene counterpart of [`crate::ui_root`]. It carries no `Name`: add one with
/// `#MyRoot` at the call site if you want to reference or query it.
pub fn ui_root() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
        }
        Pickable::IGNORE
    }
}

/// [`ui_root`] on the theme's screen background ([`ColorToken::Background`]): a
/// full-window menu screen, a title screen.
pub fn screen() -> impl Scene {
    bsn! {
        @ui_root()
        ThemedBackground(ColorToken::Background)
    }
}

/// [`ui_root`] over a dimmed backdrop ([`ColorToken::Overlay`]): pause menus, dialogs,
/// anything shown over a running game. The backdrop blocks clicks to what is under it.
pub fn backdrop() -> impl Scene {
    bsn! {
        @ui_root()
        ThemedBackground(ColorToken::Overlay)
        Pickable::default()
        GlobalZIndex(100)
    }
}

/// A full-window layer with no layout of its own: the parent for [`anchored`] groups in a
/// HUD. Clicks pass through it.
pub fn fullscreen() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
        }
        Pickable::IGNORE
    }
}

/// Where an [`anchored`] group sits on the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScreenAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    #[default]
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl ScreenAnchor {
    /// `(justify_content, align_items)` for a full-window column that puts its content at
    /// this anchor.
    const fn alignment(self) -> (JustifyContent, AlignItems) {
        use AlignItems as A;
        use JustifyContent as J;
        match self {
            Self::TopLeft => (J::FlexStart, A::FlexStart),
            Self::Top => (J::FlexStart, A::Center),
            Self::TopRight => (J::FlexStart, A::FlexEnd),
            Self::Left => (J::Center, A::FlexStart),
            Self::Center => (J::Center, A::Center),
            Self::Right => (J::Center, A::FlexEnd),
            Self::BottomLeft => (J::FlexEnd, A::FlexStart),
            Self::Bottom => (J::FlexEnd, A::Center),
            Self::BottomRight => (J::FlexEnd, A::FlexEnd),
        }
    }
}

/// A HUD corner (or edge, or centre): a full-window column that stacks its children at
/// `anchor`, `margin` pixels in from the window edge. Clicks pass through the empty
/// space, not through the children.
///
/// ```ignore
/// @scenes::fullscreen()
/// Children [
///     @scenes::anchored(ScreenAnchor::TopLeft, 16.0)  Children [ @minimap() ]
///     --
///     @scenes::anchored(ScreenAnchor::BottomRight, 16.0) Children [ @ammo() ]
/// ]
/// ```
pub fn anchored(anchor: ScreenAnchor, margin: f32) -> impl Scene {
    let (justify, align) = anchor.alignment();
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: {justify},
            align_items: {align},
            padding: {UiRect::all(px(margin))},
            row_gap: px(6),
        }
        Pickable::IGNORE
    }
}

/// A flex row.
pub fn row(column_gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            column_gap: {px(column_gap)},
        }
    }
}

/// A flex column.
pub fn column(row_gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: {px(row_gap)},
        }
    }
}

/// A padded flex column, for grouping a block of content. No background; see [`card`]
/// for a themed surface.
pub fn panel(padding: f32, row_gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::all(px(padding))},
            row_gap: {px(row_gap)},
        }
    }
}

/// A raised, bordered, rounded surface: the menu panel, a dialog, an inventory window.
/// Colours come from [`ColorToken::SurfaceRaised`] / [`ColorToken::SurfaceBorder`].
///
/// Children stretch to its width, so buttons in a card line up without each being sized.
pub fn card() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,
            padding: {UiRect::all(px(28.0))},
            row_gap: px(14),
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(14.0))},
            min_width: px(300),
        }
        ThemedBackground(ColorToken::SurfaceRaised)
        ThemedBorderColor(ColorToken::SurfaceBorder)
    }
}

/// A uniform grid of `cols` equal columns.
pub fn grid(cols: u16, gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Grid,
            grid_template_columns: {vec![RepeatedGridTrack::flex(cols, 1.0)]},
            row_gap: {px(gap)},
            column_gap: {px(gap)},
        }
    }
}

/// A table: a grid of `cols` columns, each as wide as its widest cell, so it sizes itself
/// to fit the content (long names included). Children are the cells, row by row; start
/// with a [`crate::scenes::caption`] per column for the header.
pub fn table(cols: u16, column_gap: f32, row_gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Grid,
            grid_template_columns: {vec![RepeatedGridTrack::auto(cols)]},
            column_gap: {px(column_gap)},
            row_gap: {px(row_gap)},
            align_items: AlignItems::Center,
        }
    }
}

/// A node that centers its children on both axes.
pub fn centered() -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// A `flex_grow(1)` spacer that pushes its siblings apart.
pub fn spacer() -> impl Scene {
    bsn! { Node { flex_grow: 1.0 } }
}

/// A full-height fixed-width sidebar column on the [`ColorToken::PanelBg`] surface.
pub fn side_panel(width: f32) -> impl Scene {
    bsn! {
        Node {
            width: {px(width)},
            height: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::all(px(6.0))},
            row_gap: {px(3.0)},
        }
        ThemedBackground(ColorToken::PanelBg)
    }
}

/// A full-width scrollable flex column that grows to fill the space it is given.
///
/// `ScrollPosition` is part of the widget: `Overflow::scroll_y` alone does nothing, and
/// `systems::handle_scroll_input` only sees entities that have the component. Focus moving
/// onto a child that is out of view scrolls it into view (see [`crate::focus`]).
pub fn scrollable_list(gap: f32) -> impl Scene {
    bsn! {
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            row_gap: {px(gap)},
            flex_grow: 1.0,
            width: percent(100),
            overflow: {Overflow::scroll_y()},
        }
        ScrollPosition
    }
}

/// [`scrollable_list`], capped at `max_height` instead of growing.
pub fn scrollable_list_bounded(gap: f32, max_height: f32) -> impl Scene {
    bsn! {
        @scrollable_list(gap)
        Node { flex_grow: 0.0, max_height: {px(max_height)} }
    }
}
