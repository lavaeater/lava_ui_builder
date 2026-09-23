//! Theme tokens: the indirection that lets a running app change theme without
//! rebuilding its UI.
//!
//! A scene widget does not bake a `Color` into `BackgroundColor`. It carries a token --
//! `ThemedBackground(ColorToken::ButtonBg)` -- and [`apply_theme_tokens`] resolves tokens
//! against the [`LavaTheme`] resource. Swapping themes is then one resource write:
//!
//! ```ignore
//! *theme = light_theme();   // every themed entity repaints next frame
//! ```
//!
//! This mirrors `bevy_feathers`' `ThemeBackgroundColor(tokens::BUTTON_BG)`, with one
//! difference: the token set here is an `enum` rather than a string newtype. The tokens
//! are exactly the fields of [`LavaTheme`], so an enum makes a typo a compile error and
//! the resolver exhaustive. The cost is that downstream crates cannot invent new tokens;
//! if that is ever needed, this becomes a newtype plus a map, as feathers does it.

use bevy::prelude::*;

use crate::{InteractionPalette, LavaTheme};

/// A named color in [`LavaTheme`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorToken {
    #[default]
    ButtonBg,
    ButtonBgHovered,
    ButtonBgPressed,
    ButtonText,
    ButtonBorder,
    CollapsibleBg,
    CollapsibleBgHovered,
    CollapsibleBgPressed,
    HeaderText,
    LabelText,
    PanelBg,
    PanelBorder,
}

impl ColorToken {
    /// Resolve this token against a theme.
    pub fn color(self, theme: &LavaTheme) -> Color {
        match self {
            Self::ButtonBg => theme.button.bg,
            Self::ButtonBgHovered => theme.button.bg_hovered,
            Self::ButtonBgPressed => theme.button.bg_pressed,
            Self::ButtonText => theme.button.text_color,
            Self::ButtonBorder => theme.button.border_color,
            Self::CollapsibleBg => theme.button.collapsible_bg,
            Self::CollapsibleBgHovered => theme.button.collapsible_bg_hovered,
            Self::CollapsibleBgPressed => theme.button.collapsible_bg_pressed,
            Self::HeaderText => theme.text.header_color,
            Self::LabelText => theme.text.label_color,
            Self::PanelBg => theme.bg_color,
            Self::PanelBorder => theme.border_color,
        }
    }
}

/// A named font + size pairing in [`LavaTheme`].
///
/// Colors alone are not enough to keep a widget theme-free: without this, every text
/// scene would still need the theme passed in just to learn its font size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum FontToken {
    Header,
    #[default]
    Label,
    Button,
}

impl FontToken {
    /// Resolve this token against a theme, returning `(font, size)`.
    pub fn font(self, theme: &LavaTheme) -> (Handle<Font>, f32) {
        match self {
            Self::Header => (theme.text.font.clone(), theme.text.header_size),
            Self::Label => (theme.text.font.clone(), theme.text.label_size),
            Self::Button => (theme.button.font.clone(), theme.button.font_size),
        }
    }
}

/// Paint this entity's [`BackgroundColor`] from a token.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(BackgroundColor)]
pub struct ThemedBackground(pub ColorToken);

/// Paint this entity's [`TextColor`] from a token.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TextColor)]
pub struct ThemedTextColor(pub ColorToken);

/// Paint this entity's [`BorderColor`] from a token.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(BorderColor)]
pub struct ThemedBorderColor(pub ColorToken);

/// Fill this entity's [`InteractionPalette`] from tokens.
///
/// Note the layering: this system writes the palette, and `apply_interaction_palette`
/// turns the palette into the actual `BackgroundColor` for the current hover/press state.
/// An entity with a `ThemedPalette` should not also carry a [`ThemedBackground`] -- they
/// would both claim `BackgroundColor`.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(InteractionPalette)]
pub struct ThemedPalette {
    pub none: ColorToken,
    pub hovered: ColorToken,
    pub pressed: ColorToken,
}

/// Set this entity's [`TextFont`] font and size from a token.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TextFont)]
pub struct ThemedFont(pub FontToken);

/// Resolve every theme token into its concrete component.
///
/// Runs unconditionally rather than on `resource_changed::<LavaTheme>`, so that entities
/// spawned after a theme change are painted too; every write is guarded by an equality
/// check, so change detection on the painted components stays meaningful.
pub fn apply_theme_tokens(
    theme: Option<Res<LavaTheme>>,
    mut backgrounds: Query<(&ThemedBackground, &mut BackgroundColor)>,
    mut text_colors: Query<(&ThemedTextColor, &mut TextColor)>,
    mut border_colors: Query<(&ThemedBorderColor, &mut BorderColor)>,
    mut palettes: Query<(&ThemedPalette, &mut InteractionPalette)>,
    mut fonts: Query<(&ThemedFont, &mut TextFont)>,
) {
    let Some(theme) = theme else {
        return;
    };
    let theme = &*theme;

    for (token, mut bg) in &mut backgrounds {
        let color = token.0.color(theme);
        if bg.0 != color {
            bg.0 = color;
        }
    }
    for (token, mut text_color) in &mut text_colors {
        let color = token.0.color(theme);
        if text_color.0 != color {
            text_color.0 = color;
        }
    }
    for (token, mut border) in &mut border_colors {
        let color = token.0.color(theme);
        if border.left != color {
            *border = BorderColor::all(color);
        }
    }
    for (tokens, mut palette) in &mut palettes {
        let (none, hovered, pressed) = (
            tokens.none.color(theme),
            tokens.hovered.color(theme),
            tokens.pressed.color(theme),
        );
        if palette.none != none || palette.hovered != hovered || palette.pressed != pressed {
            *palette = InteractionPalette {
                none,
                hovered,
                pressed,
            };
        }
    }
    for (token, mut text_font) in &mut fonts {
        let (font, size) = token.0.font(theme);
        let size = FontSize::from(size);
        if text_font.font != FontSource::Handle(font.clone()) || text_font.font_size != size {
            text_font.font = FontSource::Handle(font);
            text_font.font_size = size;
        }
    }
}
