//! Theme tokens: the indirection that lets a running app change theme without
//! rebuilding its UI.
//!
//! A scene widget does not bake a `Color` into `BackgroundColor`. It carries a token --
//! `ThemedBackground(ColorToken::PanelBg)` -- and [`apply_theme_tokens`] resolves tokens
//! against the [`LavaTheme`] resource. Swapping themes is then one resource write:
//!
//! ```ignore
//! *theme = LavaTheme::light();   // every themed entity repaints next frame
//! ```
//!
//! Buttons use a [`Variant`] instead of a single token: the variant picks a whole
//! [`ButtonColors`] set (three backgrounds and a caption colour), and Bevy's state markers
//! override it -- `Checked` / `Selected` paint the theme's *selected* look,
//! `InteractionDisabled` the *disabled* one. Selection and disabling are therefore one
//! marker insert, never a palette swap in a game system.
//!
//! This mirrors `bevy_feathers`' `ThemeBackgroundColor(tokens::BUTTON_BG)`, with one
//! difference: the token set here is an `enum` rather than a string newtype. The tokens
//! are exactly the fields of [`LavaTheme`], so an enum makes a typo a compile error and
//! the resolver exhaustive. The cost is that downstream crates cannot invent new tokens;
//! if that is ever needed, this becomes a newtype plus a map, as feathers does it.

use bevy::prelude::*;
use bevy::ui::{Checked, InteractionDisabled, Selected};

use crate::{ButtonColors, InteractionPalette, LavaTheme};

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
    MeterBg,
    MeterFill,
    HealthFill,
    PanelBg,
    PanelBorder,
    // ── surfaces and semantic text (`LavaTheme::surface`) ──
    Background,
    SurfaceRaised,
    SurfaceBorder,
    Overlay,
    TooltipBg,
    TooltipText,
    TextMuted,
    Accent,
    Success,
    Warning,
    Danger,
    // ── controls (`LavaTheme::controls`) ──
    Track,
    Fill,
    Thumb,
    ThumbHovered,
    CheckMark,
    InputBg,
    InputBorder,
    InputText,
    Placeholder,
    FocusRing,
}

impl ColorToken {
    /// Resolve this token against a theme.
    pub fn color(self, theme: &LavaTheme) -> Color {
        let (s, c) = (&theme.surface, &theme.controls);
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
            Self::MeterBg => theme.meter.bg,
            Self::MeterFill => theme.meter.fill,
            Self::HealthFill => theme.meter.health,
            Self::PanelBg => theme.bg_color,
            Self::PanelBorder => theme.border_color,
            Self::Background => s.background,
            Self::SurfaceRaised => s.raised,
            Self::SurfaceBorder => s.raised_border,
            Self::Overlay => s.overlay,
            Self::TooltipBg => s.tooltip_bg,
            Self::TooltipText => s.tooltip_text,
            Self::TextMuted => s.text_muted,
            Self::Accent => s.accent,
            Self::Success => s.success,
            Self::Warning => s.warning,
            Self::Danger => s.danger,
            Self::Track => c.track,
            Self::Fill => c.fill,
            Self::Thumb => c.thumb,
            Self::ThumbHovered => c.thumb_hovered,
            Self::CheckMark => c.check_mark,
            Self::InputBg => c.input_bg,
            Self::InputBorder => c.input_border,
            Self::InputText => c.input_text,
            Self::Placeholder => c.placeholder,
            Self::FocusRing => c.focus_ring,
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
    /// The one big line on a title screen.
    Title,
    /// Captions, hints, table cells, tooltips.
    Small,
}

impl FontToken {
    /// Resolve this token against a theme, returning `(font, size)`.
    pub fn font(self, theme: &LavaTheme) -> (Handle<Font>, f32) {
        let t = &theme.text;
        match self {
            Self::Header => (t.font.clone(), t.header_size),
            Self::Label => (t.font.clone(), t.label_size),
            Self::Button => (theme.button.font.clone(), theme.button.font_size),
            Self::Title => (t.font.clone(), t.title_size),
            Self::Small => (t.font.clone(), t.small_size),
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
/// An entity with a `ThemedPalette` should not also carry a [`ThemedBackground`] or a
/// [`Variant`] -- they would all claim the same components.
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

/// Which of the theme's button looks a button uses. Patch it onto any lava button:
///
/// ```ignore
/// @scenes::button("Quit") Variant::Danger
/// ```
///
/// The variant fills the button's [`InteractionPalette`] and the colour of every
/// [`VariantText`] child. Bevy's state markers on the same entity take precedence:
/// `InteractionDisabled` paints [`crate::ButtonTheme::disabled`], and `Checked` or
/// `Selected` paint [`crate::ButtonTheme::selected`].
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[require(InteractionPalette)]
pub enum Variant {
    #[default]
    Primary,
    Secondary,
    Danger,
    Success,
    /// No background until hovered: list rows, icon buttons, toolbar items.
    Ghost,
}

impl Variant {
    /// This variant's colours in `theme`, ignoring state.
    pub const fn colors(self, theme: &LavaTheme) -> ButtonColors {
        let b = &theme.button;
        match self {
            Self::Primary => b.primary(),
            Self::Secondary => b.secondary,
            Self::Danger => b.danger,
            Self::Success => b.success,
            Self::Ghost => b.ghost,
        }
    }

    /// The colours to paint given the entity's state markers.
    pub const fn resolve(
        self,
        theme: &LavaTheme,
        selected: bool,
        disabled: bool,
    ) -> ButtonColors {
        if disabled {
            theme.button.disabled
        } else if selected {
            theme.button.selected
        } else {
            self.colors(theme)
        }
    }
}

/// Marks a text entity whose colour follows the [`Variant`] (and state) of its parent:
/// the caption of a button, the label of a list row.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(TextColor)]
pub struct VariantText;

/// The system set the token resolvers run in. `apply_interaction_palette` runs after it,
/// since tokens and variants produce the palette it reads.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ThemeSystems;

/// Just the theme resolvers ([`apply_theme_tokens`]), with no window, picking or input
/// dependencies -- what a headless test needs. [`crate::LavaUiPlugin`] adds it.
pub struct ThemeTokensPlugin;

impl Plugin for ThemeTokensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_theme_tokens.in_set(ThemeSystems));
    }
}

/// Resolve every theme token and variant into its concrete component.
///
/// Runs unconditionally rather than on `resource_changed::<LavaTheme>`, so that entities
/// spawned after a theme change are painted too, and so that a state marker added or
/// removed this frame (`Checked`, `InteractionDisabled`) repaints without needing
/// removal detection; every write is guarded by an equality check, so change detection
/// on the painted components stays meaningful.
#[expect(
    clippy::type_complexity,
    clippy::too_many_arguments,
    reason = "a Bevy system: one query per token kind; aliases would only move the noise"
)]
pub fn apply_theme_tokens(
    theme: Option<Res<LavaTheme>>,
    mut backgrounds: Query<(&ThemedBackground, &mut BackgroundColor)>,
    mut text_colors: Query<(&ThemedTextColor, &mut TextColor)>,
    mut border_colors: Query<(&ThemedBorderColor, &mut BorderColor)>,
    mut palettes: Query<(&ThemedPalette, &mut InteractionPalette), Without<Variant>>,
    mut fonts: Query<(&ThemedFont, &mut TextFont)>,
    mut variants: Query<
        (
            &Variant,
            Has<Checked>,
            Has<Selected>,
            Has<InteractionDisabled>,
            &mut InteractionPalette,
        ),
        Without<ThemedPalette>,
    >,
    mut variant_texts: Query<(&ChildOf, &mut TextColor), (With<VariantText>, Without<ThemedTextColor>)>,
    variant_owners: Query<(&Variant, Has<Checked>, Has<Selected>, Has<InteractionDisabled>)>,
    parents: Query<&ChildOf>,
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
        set_palette(
            &mut palette,
            tokens.none.color(theme),
            tokens.hovered.color(theme),
            tokens.pressed.color(theme),
        );
    }
    for (token, mut text_font) in &mut fonts {
        let (font, size) = token.0.font(theme);
        let size = FontSize::from(size);
        if text_font.font != FontSource::Handle(font.clone()) || text_font.font_size != size {
            text_font.font = FontSource::Handle(font);
            text_font.font_size = size;
        }
    }
    for (variant, checked, selected, disabled, mut palette) in &mut variants {
        let colors = variant.resolve(theme, checked || selected, disabled);
        set_palette(&mut palette, colors.bg, colors.hovered, colors.pressed);
    }
    // A caption follows the nearest ancestor that has a variant: usually its parent, but
    // a row may wrap its text in a layout node.
    for (child_of, mut text_color) in &mut variant_texts {
        let owner = std::iter::once(child_of.parent())
            .chain(parents.iter_ancestors(child_of.parent()))
            .find_map(|e| variant_owners.get(e).ok());
        let Some((variant, checked, selected, disabled)) = owner else {
            continue;
        };
        let color = variant.resolve(theme, checked || selected, disabled).text;
        if text_color.0 != color {
            text_color.0 = color;
        }
    }
}

fn set_palette(palette: &mut Mut<InteractionPalette>, none: Color, hovered: Color, pressed: Color) {
    if palette.none != none || palette.hovered != hovered || palette.pressed != pressed {
        **palette = InteractionPalette {
            none,
            hovered,
            pressed,
        };
    }
}
