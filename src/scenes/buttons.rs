//! Buttons. Every button here is a `bevy::ui_widgets::Button` (it emits `Activate` on
//! click, ENTER, SPACE and gamepad A) and is focusable, so menus built from them work
//! with keyboard and controller without extra code.
//!
//! The look is a [`Variant`]; patch a different one to change it:
//!
//! ```ignore
//! @scenes::button("Quit") Variant::Danger on(quit)
//! ```
//!
//! Selection and disabling are Bevy's own markers on the button: `Selected` / `Checked`
//! paint the theme's *selected* look, `InteractionDisabled` the *disabled* one.

use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Selected;
use bevy::ui::auto_directional_navigation::AutoDirectionalNavigation;
use bevy::ui_widgets::Button as WidgetsButton;

use crate::{
    ColorToken, FontToken, InteractionPalette, ThemedBorderColor, ThemedFont, ThemedTextColor,
    Variant, VariantText,
};

/// Default button metrics. Sizes are layout, not theme: patch `Node` to change them.
const BUTTON_MIN_WIDTH: f32 = 150.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_BORDER: f32 = 2.0;
const BUTTON_PADDING_X: f32 = 24.0;

/// Makes an entity reachable by Tab and by arrow keys / D-pad / stick. Every interactive
/// lava widget includes it; patch it onto your own widgets to join the navigation.
pub fn focusable() -> impl Scene {
    bsn! {
        TabIndex(0)
        AutoDirectionalNavigation
    }
}

/// Layout and behaviour shared by every text button: size, border, the headless button,
/// hover tracking and focus. No colours.
fn button_shell() -> impl Scene {
    bsn! {
        Node {
            min_width: {px(BUTTON_MIN_WIDTH)},
            height: {px(BUTTON_HEIGHT)},
            padding: {UiRect::horizontal(px(BUTTON_PADDING_X))},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: {UiRect::all(px(BUTTON_BORDER))},
            border_radius: BorderRadius::MAX,
        }
        WidgetsButton
        Hovered
        @focusable()
        ThemedBorderColor(ColorToken::ButtonBorder)
    }
}

/// A themed button with a text caption, in the [`Variant::Primary`] look.
///
/// Attach the action at the call site with `on(..)` (it must be a `Clone` closure):
///
/// ```ignore
/// @scenes::button("Play") on(|_: On<Activate>, mut next: ResMut<NextState<Screen>>| next.set(Screen::Game))
/// ```
///
/// It is at least 150 px wide and as wide as its caption; in a [`crate::scenes::card`] it
/// stretches to the card's width. Patch `Node` to fix a size.
pub fn button(content: impl Into<String>) -> impl Scene {
    let content = content.into();
    bsn! {
        @button_shell()
        Variant::Primary
        Children [
            Text({content})
            ThemedFont(FontToken::Button)
            VariantText
            Pickable::IGNORE
        ]
    }
}

/// A button with explicit colors instead of theme tokens.
///
/// The escape hatch for a button whose colour is content rather than style. For the usual
/// cases -- a red Quit, a green Confirm -- use [`button`] with [`Variant::Danger`] /
/// [`Variant::Success`] instead, which follow a theme swap.
pub fn button_colored(
    content: impl Into<String>,
    none: Color,
    hovered: Color,
    pressed: Color,
) -> impl Scene {
    let content = content.into();
    bsn! {
        @button_shell()
        InteractionPalette { none: {none}, hovered: {hovered}, pressed: {pressed} }
        Children [
            Text({content})
            ThemedFont(FontToken::Button)
            ThemedTextColor(ColorToken::ButtonText)
            Pickable::IGNORE
        ]
    }
}

/// A small square button showing a single glyph, in the [`Variant::Secondary`] look.
pub fn icon_button(glyph: impl Into<String>, size: f32) -> impl Scene {
    let glyph = glyph.into();
    bsn! {
        Node {
            width: {px(size)},
            height: {px(size)},
            display: Display::Flex,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(size * 0.2))},
        }
        WidgetsButton
        Hovered
        @focusable()
        Variant::Secondary
        Children [
            Text({glyph})
            TextFont { font_size: {px(size * 0.6)} }
            VariantText
            Pickable::IGNORE
        ]
    }
}

/// The standard 16x16 "x" used to remove a list row, in the [`Variant::Danger`] look.
pub fn delete_button() -> impl Scene {
    bsn! {
        @icon_button("x", 16.0)
        Variant::Danger
    }
}

/// A selectable list row, in the [`Variant::Ghost`] look; `selected` starts it with
/// Bevy's `Selected` marker, which paints the theme's selected colours. Move the marker to
/// change the selection -- no palette juggling.
pub fn list_item(name: impl Into<String>, selected: bool) -> impl Scene {
    let name = name.into();
    (
        bsn! {
            Node {
                width: percent(100),
                padding: {UiRect::axes(px(8.0), px(5.0))},
                border_radius: {BorderRadius::all(px(4.0))},
            }
            WidgetsButton
            Hovered
            @focusable()
            Variant::Ghost
            Children [
                Text({name})
                ThemedFont(FontToken::Small)
                VariantText
                Pickable::IGNORE
            ]
        },
        selected.then_some(bsn! { Selected }),
    )
}
