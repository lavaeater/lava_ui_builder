//! lava_ui_builder: a game UI kit for Bevy 0.20 with a theme you can swap at runtime.
//!
//! Start with [`prelude`] and [`scenes`]: widgets are BSN scene functions, coloured by
//! theme tokens, customised by patching. See `README.md` for a tour and `roadmap.md` for
//! where the library is going.
//!
//! Three APIs live here. The **scene API** ([`scenes`]) is the one to use; the
//! bundle-function API ([`bundles`], re-exported at the root) and the imperative
//! [`UIBuilder`] are frozen and kept for existing downstream code.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::window::WindowResized;

pub mod bind;
mod builder;
mod button_builder;
pub mod bundles;
pub mod components;
pub mod controls;
pub mod crawl;
pub mod drag;
pub mod focus;
pub mod media;
pub mod overlay;
pub mod responsive;
pub mod scenes;
pub mod screens;
pub mod systems;
pub mod theme;
pub mod tokens;

pub use bundles::*;
pub use components::*;
pub use bind::BindAppExt;
pub use drag::{Draggable, DropTarget, Dropped};
pub use focus::{NavigateBack, UiNavigation};
pub use overlay::{DialogResult, Toast, ToastKind, Tooltip};
pub use responsive::{NarrowLayout, Responsive};
pub use scenes::ScreenAnchor;
pub use screens::{Back, GoTo, ScreenAppExt};
pub use theme::*;
pub use tokens::{
    ColorToken, FontToken, ThemedBackground, ThemedBorderColor, ThemedFont, ThemedPalette,
    ThemedTextColor, Variant, VariantText,
};

// Re-export feathers types for external use.
// `bevy_feathers` 0.20 dropped its imperative button template (`button_bundle`); only the BSN
// scene (`FeathersButton`) is left, which doesn't fit this crate's `Commands`-based builder.
// `ButtonProps` / `feathers_button_fn` keep the old names and build the same parts as the
// 0.20 scene, so the feathers styling systems (which key on `ButtonVariant`, `Hovered` and
// the theme components) dress it the same way.
pub use bevy::feathers::controls::ButtonVariant;
pub use bevy::feathers::rounded_corners::RoundedCorners;
pub use bevy::feathers::theme::ThemedText;
pub use bevy::ui::InteractionDisabled;
pub use bevy::ui_widgets::Activate;

/// Compiles the README's code blocks as doctests, so the front-page example cannot rot.
#[doc = include_str!("../README.md")]
#[cfg(doctest)]
pub struct ReadmeDoctests;

/// Everything a game needs to build UI with the scene API, in one import:
///
/// ```ignore
/// use bevy::prelude::*;
/// use lava_ui_builder::prelude::*;
/// ```
pub mod prelude {
    pub use crate::bind::BindAppExt;
    pub use crate::controls::{Cycle, OptionIndex, Select};
    pub use crate::drag::Dropped;
    pub use crate::focus::{NavigateBack, UiNavigation};
    pub use crate::overlay::{DialogResult, Toast, ToastKind, Tooltip};
    pub use crate::responsive::{NarrowLayout, Responsive};
    pub use crate::scenes::{self, ScreenAnchor};
    pub use crate::screens::{Back, GoTo, ScreenAppExt};
    pub use crate::theme::{
        ButtonColors, ButtonTheme, ControlTheme, LavaTheme, MeterTheme, SurfaceTheme, TextTheme,
    };
    pub use crate::tokens::{
        ColorToken, FontToken, ThemedBackground, ThemedBorderColor, ThemedFont, ThemedPalette,
        ThemedTextColor, Variant,
    };
    pub use crate::{LavaUiPlugin, ProgressBar, WorldFollower};
    pub use bevy::input_focus::AutoFocus;
    pub use bevy::text::EditableText;
    pub use bevy::ui::{Checked, InteractionDisabled, Selected};
    pub use bevy::ui_widgets::{Activate, ValueChange};
}

// ============================================================================
// Imperative UIBuilder — frozen; kept for existing downstream code.
// ============================================================================

/// Fluent UI Builder for creating Bevy UI elements imperatively.
///
/// **Frozen.** Downstream projects still use it, so it keeps working, but it gets no new
/// widgets: use [`scenes`] for new UI.
pub struct UIBuilder<'w, 's> {
    commands: Commands<'w, 's>,
    theme: LavaTheme,
    root_entity: Entity,
    current_entity: Entity,
    parent_stack: VecDeque<Entity>,
}

/// Builder for button-specific properties, wraps UIBuilder.
pub struct ButtonBuilder<'a, 'w, 's> {
    ui: &'a mut UIBuilder<'w, 's>,
    text_entity: Option<Entity>,
}

// ============================================================================
// Plugin
// ============================================================================

/// Everything the library needs at runtime: theme tokens and variants, interaction
/// palettes, the form controls, keyboard / gamepad navigation and the focus ring,
/// tooltips / toasts / dialogs, collapsibles, scrolling, meters, responsive layout,
/// world followers and UI scaling.
///
/// Bindings ([`bind`]) and screens ([`screens`]) are generic over your types, so they are
/// registered per type: `app.bind_resource::<Hud>()`, `app.add_screen(Menu::Main, menu)`.
pub struct LavaUiPlugin;

impl Plugin for LavaUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiScale(1.0))
            .add_plugins((
                tokens::ThemeTokensPlugin,
                controls::ControlsPlugin,
                focus::FocusPlugin,
                overlay::OverlayPlugin,
                drag::DragPlugin,
            ))
            // Collapsing is driven by the `Activate` event, so it is an observer rather
            // than a system in the `Update` set.
            .add_observer(systems::toggle_collapsible_on_activate)
            .add_systems(
                Update,
                (
                    systems::apply_interaction_palette.after(tokens::ThemeSystems),
                    systems::handle_scroll_input,
                    systems::update_collapsible_visibility,
                    systems::sync_progress_bars,
                    responsive::apply_responsive,
                    adapt_ui_scale,
                ),
            )
            .add_systems(
                PostUpdate,
                systems::world_follower_system.before(TransformSystems::Propagate),
            );
    }
}

fn adapt_ui_scale(
    mut message_reader: MessageReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
    theme: Option<Res<LavaTheme>>,
) {
    for event in message_reader.read() {
        let base_width = theme.as_ref().map_or(1920.0, |t| t.ui_width);
        ui_scale.0 = (event.width / base_width).max(0.5);
    }
}
