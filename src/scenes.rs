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
    CollapseToggleButton, Collapsible, CollapsibleContent, ColorToken, FontToken, ProgressBar,
    ProgressBarFill, ThemedBorderColor, ThemedFont, ThemedPalette, ThemedTextColor,
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
