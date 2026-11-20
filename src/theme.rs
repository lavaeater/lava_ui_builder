//! The theme: every colour and font a lava widget uses, in one swappable resource.
//!
//! Widgets never read these fields directly. They carry tokens ([`crate::tokens`]) that
//! `apply_theme_tokens` resolves every frame, so replacing the resource repaints live UI:
//!
//! ```ignore
//! *theme = LavaTheme::light();                       // a preset
//! *theme = LavaTheme::from_accent(Color::srgb(0.9, 0.3, 0.5)); // one colour, derived
//! theme.button.danger.bg = Color::srgb(0.7, 0.1, 0.1);         // or field by field
//! ```
//!
//! **Adding a field is not a breaking change**: downstream themes are built with
//! `..LavaTheme::default()` / `..t.clone()`. Removing or renaming one is.
//!
//! Sizes are deliberately *not* here (beyond font sizes): they are layout, and the scene
//! API answers layout by patching `Node` at the call site. See `roadmap.md` for why a
//! size token cannot coexist with patching.

use bevy::color::palettes::basic::WHITE;
use bevy::color::{Luminance, Mix};
use bevy::prelude::*;

/// The four colours of one button look: its background in each interaction state, and
/// the caption colour on top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonColors {
    pub bg: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub text: Color,
}

impl ButtonColors {
    /// Derive hover/press shades from a base colour: hover a little lighter, press a
    /// little darker. The usual way to make a variant from one brand colour.
    pub fn from_base(bg: Color, text: Color) -> Self {
        Self {
            bg,
            hovered: bg.lighter(0.06),
            pressed: bg.darker(0.06),
            text,
        }
    }

    /// The same colour in every state: for disabled buttons, which must not react.
    pub const fn flat(bg: Color, text: Color) -> Self {
        Self {
            bg,
            hovered: bg,
            pressed: bg,
            text,
        }
    }
}

/// Theme-level colors and fonts for buttons.
///
/// The flat `bg` / `bg_hovered` / `bg_pressed` / `text_color` fields are the **primary**
/// look (they predate variants, and the old bundle/builder APIs read them). The other
/// looks are picked per button with a [`crate::Variant`] component.
#[derive(Clone, Debug)]
pub struct ButtonTheme {
    pub bg: Color,
    pub bg_hovered: Color,
    pub bg_pressed: Color,
    pub text_color: Color,
    pub font: Handle<Font>,
    pub font_size: f32,
    /// Used by the bundle/builder APIs. Scene buttons are sized by patching `Node`.
    pub border_radius: BorderRadius,
    /// Used by the bundle/builder APIs. Scene buttons are sized by patching `Node`.
    pub border_width: Val,
    pub border_color: Color,
    /// Used by the bundle/builder APIs. Scene buttons are sized by patching `Node`.
    pub height: Val,
    /// Used by the bundle/builder APIs. Scene buttons are sized by patching `Node`.
    pub width: Val,
    /// Colors for collapsible section toggle buttons.
    pub collapsible_bg: Color,
    pub collapsible_bg_hovered: Color,
    pub collapsible_bg_pressed: Color,
    /// [`crate::Variant::Secondary`]: the quieter button next to a primary one.
    pub secondary: ButtonColors,
    /// [`crate::Variant::Danger`]: quit, delete, abandon.
    pub danger: ButtonColors,
    /// [`crate::Variant::Success`]: confirm, buy, accept.
    pub success: ButtonColors,
    /// [`crate::Variant::Ghost`]: no background until hovered; list rows, icon buttons.
    pub ghost: ButtonColors,
    /// Any variant while it carries Bevy's `Checked` or `Selected` marker: the chosen
    /// radio option, the active tab, the selected list row.
    pub selected: ButtonColors,
    /// Any variant while it carries `InteractionDisabled`.
    pub disabled: ButtonColors,
}

impl ButtonTheme {
    /// The primary look as a [`ButtonColors`], from the flat fields.
    pub const fn primary(&self) -> ButtonColors {
        ButtonColors {
            bg: self.bg,
            hovered: self.bg_hovered,
            pressed: self.bg_pressed,
            text: self.text_color,
        }
    }

    /// Set the primary look from a [`ButtonColors`].
    pub const fn set_primary(&mut self, colors: ButtonColors) {
        self.bg = colors.bg;
        self.bg_hovered = colors.hovered;
        self.bg_pressed = colors.pressed;
        self.text_color = colors.text;
    }
}

impl Default for ButtonTheme {
    fn default() -> Self {
        let light_text = Color::srgb(0.93, 0.93, 0.97);
        Self {
            bg: Color::srgb(0.275, 0.400, 0.750),
            bg_hovered: Color::srgb(0.384, 0.600, 0.820),
            bg_pressed: Color::srgb(0.239, 0.286, 0.600),
            text_color: Color::from(WHITE),
            font: Handle::default(),
            font_size: 40.0,
            border_radius: BorderRadius::MAX,
            border_width: px(2.0),
            border_color: Color::srgb(0.2, 0.2, 0.2),
            height: px(50.0),
            width: px(150.0),
            collapsible_bg: Color::srgb(0.25, 0.25, 0.3),
            collapsible_bg_hovered: Color::srgb(0.35, 0.35, 0.4),
            collapsible_bg_pressed: Color::srgb(0.4, 0.6, 0.4),
            secondary: ButtonColors {
                bg: Color::srgb(0.20, 0.21, 0.30),
                hovered: Color::srgb(0.27, 0.28, 0.40),
                pressed: Color::srgb(0.15, 0.15, 0.24),
                text: light_text,
            },
            danger: ButtonColors {
                bg: Color::srgb(0.55, 0.13, 0.13),
                hovered: Color::srgb(0.68, 0.18, 0.18),
                pressed: Color::srgb(0.42, 0.09, 0.09),
                text: light_text,
            },
            success: ButtonColors {
                bg: Color::srgb(0.18, 0.52, 0.27),
                hovered: Color::srgb(0.23, 0.63, 0.34),
                pressed: Color::srgb(0.13, 0.40, 0.20),
                text: light_text,
            },
            ghost: ButtonColors {
                bg: Color::srgba(1.0, 1.0, 1.0, 0.0),
                hovered: Color::srgba(1.0, 1.0, 1.0, 0.08),
                pressed: Color::srgba(1.0, 1.0, 1.0, 0.14),
                text: Color::srgb(0.80, 0.82, 0.90),
            },
            selected: ButtonColors {
                bg: Color::srgb(0.20, 0.60, 0.30),
                hovered: Color::srgb(0.26, 0.70, 0.38),
                pressed: Color::srgb(0.16, 0.48, 0.24),
                text: Color::from(WHITE),
            },
            disabled: ButtonColors::flat(
                Color::srgba(0.30, 0.30, 0.34, 0.60),
                Color::srgb(0.55, 0.55, 0.60),
            ),
        }
    }
}

/// Theme-level colors and fonts for text labels.
#[derive(Clone, Debug)]
pub struct TextTheme {
    pub font: Handle<Font>,
    pub header_size: f32,
    pub label_size: f32,
    pub header_color: Color,
    pub label_color: Color,
    /// [`crate::FontToken::Title`]: the one big line on a title screen.
    pub title_size: f32,
    /// [`crate::FontToken::Small`]: captions, hints, table cells, tooltips.
    pub small_size: f32,
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            font: Handle::default(),
            header_size: 40.0,
            label_size: 24.0,
            header_color: Color::srgb(0.988, 0.984, 0.800),
            label_color: Color::srgb(0.867, 0.827, 0.412),
            title_size: 64.0,
            small_size: 16.0,
        }
    }
}

/// Theme-level colors for meters: progress bars, health bars, charge gauges.
///
/// `fill` is the general-purpose fill; `health` exists because a health bar is the one
/// meter nearly every game has, and it usually wants its own colour regardless of what the
/// rest of the palette does.
#[derive(Clone, Debug)]
pub struct MeterTheme {
    /// The empty part of the bar, behind the fill.
    pub bg: Color,
    pub fill: Color,
    pub health: Color,
}

impl Default for MeterTheme {
    fn default() -> Self {
        Self {
            bg: Color::srgba(0.0, 0.0, 0.0, 0.55),
            fill: Color::srgb(0.275, 0.600, 0.900),
            health: Color::srgb(0.200, 0.800, 0.300),
        }
    }
}

/// Surfaces and semantic text colours: what panels, overlays and status text use.
///
/// The `success` / `warning` / `danger` / `accent` colours are for *text and small marks*
/// (a red low-ammo counter, a green "connected"): they are what keeps a HUD on the theme
/// instead of hardcoding `Color::srgb(1.0, 0.2, 0.2)`. Buttons have their own variant
/// colours in [`ButtonTheme`].
#[derive(Clone, Debug)]
pub struct SurfaceTheme {
    /// The screen behind everything: a full-window menu's backdrop.
    pub background: Color,
    /// A raised card or dialog: the menu panel, a popup list.
    pub raised: Color,
    /// The border around a raised surface.
    pub raised_border: Color,
    /// The scrim behind a modal dialog.
    pub overlay: Color,
    pub tooltip_bg: Color,
    pub tooltip_text: Color,
    /// Secondary text: hints, captions, column headings.
    pub text_muted: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl Default for SurfaceTheme {
    fn default() -> Self {
        Self {
            background: Color::srgb(0.05, 0.05, 0.08),
            raised: Color::srgba(0.08, 0.08, 0.14, 0.97),
            raised_border: Color::srgb(0.25, 0.25, 0.50),
            overlay: Color::srgba(0.0, 0.0, 0.0, 0.60),
            tooltip_bg: Color::srgba(0.04, 0.04, 0.07, 0.95),
            tooltip_text: Color::srgb(0.95, 0.95, 0.97),
            text_muted: Color::srgb(0.62, 0.62, 0.74),
            accent: Color::srgb(0.40, 0.75, 1.00),
            success: Color::srgb(0.35, 0.85, 0.45),
            warning: Color::srgb(0.98, 0.75, 0.25),
            danger: Color::srgb(0.95, 0.35, 0.30),
        }
    }
}

/// Colours for interactive controls: sliders, checkboxes, text inputs, the focus ring.
#[derive(Clone, Debug)]
pub struct ControlTheme {
    /// The empty track of a slider, the box of an unchecked checkbox.
    pub track: Color,
    /// The filled part of a slider, the knob of an "on" toggle.
    pub fill: Color,
    pub thumb: Color,
    pub thumb_hovered: Color,
    /// The tick inside a checked checkbox, the dot in a chosen radio.
    pub check_mark: Color,
    pub input_bg: Color,
    pub input_border: Color,
    pub input_text: Color,
    pub placeholder: Color,
    pub cursor: Color,
    pub selection: Color,
    /// The outline drawn around whatever has keyboard / gamepad focus.
    pub focus_ring: Color,
}

impl Default for ControlTheme {
    fn default() -> Self {
        Self {
            track: Color::srgb(0.16, 0.17, 0.24),
            fill: Color::srgb(0.275, 0.400, 0.750),
            thumb: Color::srgb(0.92, 0.92, 0.96),
            thumb_hovered: Color::from(WHITE),
            check_mark: Color::srgb(0.40, 0.75, 1.00),
            input_bg: Color::srgb(0.07, 0.07, 0.11),
            input_border: Color::srgb(0.30, 0.30, 0.45),
            input_text: Color::srgb(0.95, 0.95, 0.97),
            placeholder: Color::srgb(0.48, 0.48, 0.56),
            cursor: Color::srgb(0.95, 0.95, 1.00),
            selection: Color::srgba(0.40, 0.75, 1.00, 0.35),
            focus_ring: Color::srgb(1.00, 0.84, 0.35),
        }
    }
}

/// Top-level UI theme resource. Every themed lava widget resolves its colours and fonts
/// from this; replace or mutate it to restyle live UI.
#[derive(Resource, Clone, Debug)]
pub struct LavaTheme {
    pub button: ButtonTheme,
    pub text: TextTheme,
    pub meter: MeterTheme,
    pub surface: SurfaceTheme,
    pub controls: ControlTheme,
    /// [`crate::ColorToken::PanelBg`]: the plain panel behind a block of content.
    pub bg_color: Color,
    /// [`crate::ColorToken::PanelBorder`].
    pub border_color: Color,
    pub ui_width: f32,
    pub ui_height: f32,
}

impl Default for LavaTheme {
    fn default() -> Self {
        Self {
            button: ButtonTheme::default(),
            text: TextTheme::default(),
            meter: MeterTheme::default(),
            surface: SurfaceTheme::default(),
            controls: ControlTheme::default(),
            bg_color: Color::srgba(0.5, 0.5, 0.5, 0.9),
            border_color: Color::srgba(0.2, 0.2, 0.2, 0.9),
            ui_width: 1920.0,
            ui_height: 1080.0,
        }
    }
}

impl LavaTheme {
    /// The default theme: dark surfaces, blue buttons, warm text.
    pub fn dark() -> Self {
        Self::default()
    }

    /// A light theme: pale surfaces, dark text, the same structure as [`Self::dark`].
    pub fn light() -> Self {
        let ink = Color::srgb(0.10, 0.10, 0.16);
        let soft_ink = Color::srgb(0.25, 0.25, 0.34);
        let mut theme = Self::default();
        theme.text.header_color = ink;
        theme.text.label_color = soft_ink;
        theme.button.set_primary(ButtonColors::from_base(
            Color::srgb(0.22, 0.45, 0.85),
            Color::from(WHITE),
        ));
        theme.button.border_color = Color::srgb(0.70, 0.72, 0.80);
        theme.button.collapsible_bg = Color::srgb(0.85, 0.86, 0.90);
        theme.button.collapsible_bg_hovered = Color::srgb(0.78, 0.80, 0.86);
        theme.button.collapsible_bg_pressed = Color::srgb(0.70, 0.82, 0.72);
        theme.button.secondary = ButtonColors::from_base(Color::srgb(0.86, 0.87, 0.92), ink);
        theme.button.danger =
            ButtonColors::from_base(Color::srgb(0.80, 0.22, 0.22), Color::from(WHITE));
        theme.button.success =
            ButtonColors::from_base(Color::srgb(0.20, 0.62, 0.32), Color::from(WHITE));
        theme.button.ghost = ButtonColors {
            bg: Color::srgba(0.0, 0.0, 0.0, 0.0),
            hovered: Color::srgba(0.0, 0.0, 0.0, 0.06),
            pressed: Color::srgba(0.0, 0.0, 0.0, 0.12),
            text: soft_ink,
        };
        theme.button.selected =
            ButtonColors::from_base(Color::srgb(0.95, 0.68, 0.20), Color::srgb(0.15, 0.10, 0.02));
        theme.button.disabled = ButtonColors::flat(
            Color::srgba(0.80, 0.80, 0.84, 0.80),
            Color::srgb(0.55, 0.55, 0.60),
        );
        theme.meter.bg = Color::srgba(0.0, 0.0, 0.0, 0.12);
        theme.surface = SurfaceTheme {
            background: Color::srgb(0.90, 0.91, 0.95),
            raised: Color::srgba(1.0, 1.0, 1.0, 0.97),
            raised_border: Color::srgb(0.75, 0.76, 0.84),
            overlay: Color::srgba(0.0, 0.0, 0.0, 0.35),
            tooltip_bg: Color::srgba(0.12, 0.12, 0.18, 0.95),
            tooltip_text: Color::srgb(0.97, 0.97, 0.99),
            text_muted: Color::srgb(0.42, 0.42, 0.52),
            accent: Color::srgb(0.15, 0.45, 0.85),
            success: Color::srgb(0.12, 0.55, 0.25),
            warning: Color::srgb(0.75, 0.48, 0.05),
            danger: Color::srgb(0.80, 0.18, 0.15),
        };
        theme.controls = ControlTheme {
            track: Color::srgb(0.84, 0.85, 0.90),
            fill: Color::srgb(0.22, 0.45, 0.85),
            thumb: Color::from(WHITE),
            thumb_hovered: Color::srgb(0.94, 0.95, 1.0),
            check_mark: Color::srgb(0.22, 0.45, 0.85),
            input_bg: Color::from(WHITE),
            input_border: Color::srgb(0.70, 0.72, 0.80),
            input_text: ink,
            placeholder: Color::srgb(0.58, 0.58, 0.66),
            cursor: ink,
            selection: Color::srgba(0.22, 0.45, 0.85, 0.30),
            focus_ring: Color::srgb(0.95, 0.55, 0.10),
        };
        theme.bg_color = Color::srgba(0.93, 0.94, 0.97, 0.95);
        theme.border_color = Color::srgba(0.70, 0.72, 0.80, 0.9);
        theme
    }

    /// A dark theme built around one colour: primary buttons, slider fills, checks,
    /// selection highlight, the accent and the meter fill all derive from `accent`.
    ///
    /// The caption on primary buttons turns dark when the accent is light, so any
    /// colour reads.
    pub fn from_accent(accent: Color) -> Self {
        let on_accent = if accent.luminance() > 0.6 {
            Color::srgb(0.08, 0.08, 0.10)
        } else {
            Color::from(WHITE)
        };
        let mut theme = Self::default();
        theme.button.set_primary(ButtonColors::from_base(accent, on_accent));
        theme.button.selected = ButtonColors::from_base(accent.lighter(0.12), on_accent);
        theme.button.border_color = accent.darker(0.15);
        theme.surface.accent = accent.lighter(0.10);
        theme.surface.background = accent.darker(0.45).mix(&Color::BLACK, 0.6);
        theme.surface.raised_border = accent.darker(0.10).mix(&theme.surface.raised, 0.5);
        theme.meter.fill = accent;
        theme.controls.fill = accent;
        theme.controls.check_mark = accent.lighter(0.10);
        theme.controls.selection = accent.with_alpha(0.35);
        theme.controls.focus_ring = accent.lighter(0.25);
        theme
    }

    /// Use `font` for every text role (headers, labels, buttons). Chainable:
    /// `LavaTheme::light().with_font(handle)`.
    #[must_use]
    pub fn with_font(mut self, font: Handle<Font>) -> Self {
        self.text.font = font.clone();
        self.button.font = font;
        self
    }
}
