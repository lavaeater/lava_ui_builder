//! BSN scene functions — the declarative half of the library.
//!
//! Each function here returns an `impl Scene`, so they compose inside `bsn!` and can be
//! patched by the caller without the widget having to expose an option for every field:
//!
//! ```ignore
//! commands.spawn_scene(bsn! {
//!     scenes::ui_root()
//!     Children [
//!         scenes::header("Main Menu"),
//!         // same widget, one field overridden
//!         scenes::label("subtitle") TextColor(Color::WHITE),
//!         (
//!             scenes::button("Play")
//!             on(|_: On<Activate>| info!("play"))
//!         ),
//!     ]
//! });
//! ```
//!
//! Widgets here take no theme argument. Colors and fonts are [`crate::tokens`] resolved
//! against the `LavaTheme` resource each frame, so a theme swap repaints live UI instead
//! of forcing a rebuild. A caller that wants a one-off color just patches the concrete
//! component over the token's output:
//!
//! ```ignore
//! scenes::label("danger") ThemedTextColor(ColorToken::HeaderText)  // a different token
//! scenes::label("danger") TextColor(Color::srgb(1.0, 0.2, 0.2))    // ... or no token
//! ```
//!
//! Note the second form is only stable if the entity has no `ThemedTextColor`, since the
//! token system would repaint over it. Prefer picking a different token.
//!
//! Two rules when writing scenes in here:
//!
//! * `Scene` is `Send + Sync + 'static`, so a scene may not borrow anything. Copy values
//!   into locals *before* the `bsn!` block.
//! * anything that is not BSN value syntax (a literal, a path, a struct/tuple literal or
//!   a `#Name`) has to be wrapped in braces: `Text({text})`, not `Text(text.into())`.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button as WidgetsButton;

use crate::{
    CollapseToggleButton, Collapsible, CollapsibleContent, ColorToken, FontToken,
    InteractionPalette, ProgressBar, ProgressBarFill, ThemedBorderColor, ThemedFont,
    ThemedPalette, ThemedTextColor,
};

/// Default button metrics.
///
/// Colors and fonts come from [`crate::LavaTheme`] through tokens, but *sizes* do not:
/// they are layout, and BSN already has a better answer for layout than a theme field --
/// the caller patches `Node`. `scenes::button("Play") Node { width: px(220) }` reads
/// better than a theme round-trip, and unlike a color it does not need to change at
/// runtime when the theme does.
const BUTTON_WIDTH: f32 = 150.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_BORDER: f32 = 2.0;

// Colors for widgets whose look is state, not theme -- see `list_item`.
const SECTION_LABEL_SIZE: f32 = 13.0;
const SECTION_LABEL_COLOR: Color = Color::srgb(0.5, 0.8, 0.6);
const LIST_ITEM_TEXT_SIZE: f32 = 11.0;
const LIST_ITEM_BG: Color = Color::srgba(0.07, 0.12, 0.09, 0.85);
const LIST_ITEM_HOVERED: Color = Color::srgba(0.20, 0.50, 0.28, 0.95);
const LIST_ITEM_PRESSED: Color = Color::srgba(0.12, 0.32, 0.18, 1.0);
const LIST_ITEM_TEXT: Color = Color::srgb(0.65, 0.80, 0.65);
const LIST_ITEM_SELECTED_BG: Color = Color::srgba(0.15, 0.40, 0.20, 0.95);
const LIST_ITEM_SELECTED_HOVERED: Color = Color::srgba(0.20, 0.55, 0.30, 0.95);
const LIST_ITEM_SELECTED_TEXT: Color = Color::srgb(0.9, 1.0, 0.9);
const ICON_BUTTON_TEXT_SIZE: f32 = 12.0;
const ICON_BUTTON_BG: Color = Color::srgba(0.25, 0.25, 0.30, 0.85);
const ICON_BUTTON_HOVERED: Color = Color::srgba(0.40, 0.40, 0.50, 0.95);
const ICON_BUTTON_PRESSED: Color = Color::srgba(0.20, 0.20, 0.25, 1.0);
const ICON_BUTTON_TEXT: Color = Color::srgb(0.9, 0.9, 0.9);
const DELETE_BUTTON_BG: Color = Color::srgba(0.4, 0.1, 0.1, 0.8);
const DELETE_BUTTON_HOVERED: Color = Color::srgba(0.6, 0.15, 0.15, 0.9);
const DELETE_BUTTON_PRESSED: Color = Color::srgba(0.3, 0.08, 0.08, 1.0);

/// A root UI node that fills the window and centers its content.
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
        template_value(Pickable::IGNORE)
    }
}

/// A header label: theme header font and color.
pub fn header(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Header)
        ThemedTextColor(ColorToken::HeaderText)
    }
}

/// A body label: theme label font and color.
pub fn label(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Label)
        ThemedTextColor(ColorToken::LabelText)
    }
}

/// A themed button with a text child.
///
/// The button emits [`bevy::ui_widgets::Activate`]; attach the handler at the call site
/// with `on(..)` rather than passing it in, so the observer stays next to the button it
/// belongs to. Note that `on(..)` requires the observer system to be `Clone`.
///
/// Size comes from [`BUTTON_WIDTH`]/[`BUTTON_HEIGHT`]; patch `Node` to change it.
pub fn button(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Node {
            width: {px(BUTTON_WIDTH)},
            height: {px(BUTTON_HEIGHT)},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: {UiRect::all(px(BUTTON_BORDER))},
            border_radius: BorderRadius::MAX,
        }
        WidgetsButton
        Hovered
        ThemedPalette {
            none: ColorToken::ButtonBg,
            hovered: ColorToken::ButtonBgHovered,
            pressed: ColorToken::ButtonBgPressed,
        }
        ThemedBorderColor(ColorToken::ButtonBorder)
        Children [(
            Text({text})
            ThemedFont(FontToken::Button)
            ThemedTextColor(ColorToken::ButtonText)
            template_value(Pickable::IGNORE)
        )]
    }
}

/// A horizontal progress bar. Drive it by writing to the [`ProgressBar`] component;
/// `sync_progress_bars` resizes the fill node.
pub fn progress_bar(value: f32, width: f32, height: f32, fill: Color, bg: Color) -> impl Scene {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    bsn! {
        ProgressBar { value: {value}, fill_color: {fill} }
        Node { width: {px(width)}, height: {px(height)}, overflow: {Overflow::clip()} }
        BackgroundColor({bg})
        Children [(
            ProgressBarFill
            Node { width: {percent(pct)}, height: percent(100) }
            BackgroundColor({fill})
        )]
    }
}

/// A collapsible section: a toggle button plus a content container that hides when
/// collapsed.
///
/// The `#Section` name ties the three entities together. Both `CollapseToggleButton` and
/// `CollapsibleContent` hold an `Entity`, and a `#Name` only resolves inside the `bsn!`
/// invocation that declares it -- which is why the whole widget is one scene rather than
/// three the caller assembles.
pub fn collapsible(
    label_text: impl Into<String>,
    collapsed: bool,
    content: impl SceneList,
) -> impl Scene {
    let label_text = label_text.into();
    let arrow = if collapsed { "\u{25b6}" } else { "\u{25bc}" };
    let toggle_text = format!("{arrow} {label_text}");
    let display = if collapsed {
        Display::None
    } else {
        Display::Flex
    };
    bsn! {
        #Section
        Collapsible { collapsed: {collapsed}, label: {label_text} }
        Node { flex_direction: FlexDirection::Column }
        Children [
            (
                CollapseToggleButton { target: #Section }
                WidgetsButton
                Hovered
                ThemedPalette {
                    none: ColorToken::CollapsibleBg,
                    hovered: ColorToken::CollapsibleBgHovered,
                    pressed: ColorToken::CollapsibleBgPressed,
                }
                Node {
                    padding: {UiRect::all(px(8.0))},
                    margin: {UiRect::bottom(px(4.0))},
                }
                Children [(
                    Text({toggle_text})
                    ThemedFont(FontToken::Label)
                    ThemedTextColor(ColorToken::ButtonText)
                    template_value(Pickable::IGNORE)
                )]
            ),
            (
                CollapsibleContent { parent: #Section }
                Node {
                    flex_direction: FlexDirection::Column,
                    width: percent(100),
                    display: {display},
                }
                Children [ {content} ]
            ),
        ]
    }
}

// ============================================================================
// Text that is not theme-driven
// ============================================================================

/// A text node with an explicit size and color, carrying **no** theme tokens.
///
/// Use this for text whose color is part of the content rather than part of the theme --
/// a red ammo counter, a green "connected" badge. Because there is no token, a theme swap
/// leaves it alone, and unlike [`label`] a patched `TextColor` on it stays patched.
pub fn text(content: impl Into<String>, size: f32, color: Color) -> impl Scene {
    let content = content.into();
    bsn! {
        Text({content})
        TextFont { font_size: {px(size)} }
        TextColor({color})
    }
}

/// A dim section-divider label, "-- Title --" style.
pub fn section_label(content: impl Into<String>) -> impl Scene {
    text(content, SECTION_LABEL_SIZE, SECTION_LABEL_COLOR)
}

// ============================================================================
// Layout
// ============================================================================
//
// These set up a `Node` and nothing else; the caller supplies `Children [ .. ]`.
// They exist because the flex/grid incantations are easy to get subtly wrong, not
// because BSN needs them -- `scenes::row()` is exactly `Node { flex_direction: Row }`.

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

/// A padded flex column, for grouping a block of content.
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

/// A full-height fixed-width sidebar column.
pub fn side_panel(width: f32, bg: Color) -> impl Scene {
    bsn! {
        Node {
            width: {px(width)},
            height: percent(100),
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            padding: {UiRect::all(px(6.0))},
            row_gap: {px(3.0)},
        }
        BackgroundColor({bg})
    }
}

/// A full-width scrollable flex column that grows to fill the space it is given.
///
/// `ScrollPosition` is part of the widget: `Overflow::scroll_y` alone does nothing, and
/// `systems::handle_scroll_input` only sees entities that have the component.
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
        scrollable_list(gap)
        Node { flex_grow: 0.0, max_height: {px(max_height)} }
    }
}

// ============================================================================
// Small interactive widgets
// ============================================================================

/// A selectable list row. Emits `Activate`; attach the handler with `on(..)`.
///
/// The selected/unselected colors are constants rather than theme tokens: they mark
/// *state*, not style, and the builder API they mirror hardcodes them too. Promoting
/// them to tokens would mean new `LavaTheme` fields, which is a breaking change for
/// anyone constructing the theme without `..Default::default()`.
pub fn list_item(name: impl Into<String>, selected: bool) -> impl Scene {
    let name = name.into();
    let (bg, hovered, pressed, fg) = if selected {
        (
            LIST_ITEM_SELECTED_BG,
            LIST_ITEM_SELECTED_HOVERED,
            LIST_ITEM_PRESSED,
            LIST_ITEM_SELECTED_TEXT,
        )
    } else {
        (
            LIST_ITEM_BG,
            LIST_ITEM_HOVERED,
            LIST_ITEM_PRESSED,
            LIST_ITEM_TEXT,
        )
    };
    bsn! {
        Node {
            width: percent(100),
            padding: {UiRect::axes(px(6.0), px(4.0))},
            border_radius: {BorderRadius::all(px(3.0))},
        }
        WidgetsButton
        Hovered
        template_value(InteractionPalette { none: bg, hovered, pressed })
        Children [(
            text(name, LIST_ITEM_TEXT_SIZE, fg)
            template_value(Pickable::IGNORE)
        )]
    }
}

/// A small square button showing a single glyph. Emits `Activate`.
pub fn icon_button(glyph: impl Into<String>, size: f32) -> impl Scene {
    let glyph = glyph.into();
    bsn! {
        Node {
            width: {px(size)},
            height: {px(size)},
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(3.0))},
        }
        WidgetsButton
        Hovered
        template_value(InteractionPalette {
            none: ICON_BUTTON_BG,
            hovered: ICON_BUTTON_HOVERED,
            pressed: ICON_BUTTON_PRESSED,
        })
        Children [(
            text(glyph, ICON_BUTTON_TEXT_SIZE, ICON_BUTTON_TEXT)
            template_value(Pickable::IGNORE)
        )]
    }
}

/// The standard 16x16 red "x" used to remove a list row. Emits `Activate`.
pub fn delete_button() -> impl Scene {
    bsn! {
        icon_button("x", 16.0)
        template_value(InteractionPalette {
            none: DELETE_BUTTON_BG,
            hovered: DELETE_BUTTON_HOVERED,
            pressed: DELETE_BUTTON_PRESSED,
        })
    }
}

// ============================================================================
// Runtime rebuilds
// ============================================================================

/// Replace `parent`'s children with a freshly built [`SceneList`].
///
/// The scene-API answer to `UIBuilder::start_from_entity(.., clear_children: true)`: the
/// parent entity survives, so anything referring to it (a marker query, a stored
/// `Entity`, a `WorldFollower` target) keeps working, and only the contents are rebuilt.
///
/// ```ignore
/// scenes::replace_children(&mut commands, list_root, bsn_list![
///     scenes::list_item("one", false),
///     scenes::list_item("two", true),
/// ]);
/// ```
///
/// The new children are spawned in the `SpawnScene` schedule, i.e. after `Update` in the
/// same frame -- so a system that rebuilds a list cannot also query the result in that
/// same run.
pub fn replace_children(commands: &mut Commands, parent: Entity, children: impl SceneList) {
    commands
        .entity(parent)
        .despawn_related::<Children>()
        .queue_spawn_related_scenes::<Children>(children);
}
