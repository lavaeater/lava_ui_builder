//! The bundle-function API: free functions returning `impl Bundle`, composed with Bevy's
//! `children![]`.
//!
//! **Frozen.** Kept working because downstream projects use it, but new widgets go into
//! [`crate::scenes`] only. These functions take the theme as an argument and bake colours
//! in at spawn time, so they do not follow a live theme swap the way scene widgets do.

use std::borrow::Cow;

use bevy::ecs::observer::IntoEntityObserver;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::IntoObserverSystem;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::{ButtonTheme, ButtonVariant, InteractionPalette, ProgressBar, ProgressBarFill, RoundedCorners, TextTheme};

/// What a feathers button looks like (the imperative counterpart of `FeathersButtonProps`).
#[derive(Debug, Clone, Default)]
pub struct ButtonProps {
    pub variant: ButtonVariant,
    pub corners: RoundedCorners,
}

/// A feathers button, as the 0.20 `FeathersButton` scene builds it: `overrides` are added
/// on top, `caption` spawns the label. The font is Bevy's default (the scene loads
/// feathers' own by path, which needs an asset server).
pub fn feathers_button_fn<B: Bundle, C: bevy::ecs::spawn::SpawnableList<ChildOf> + Send + Sync + 'static>(
    props: ButtonProps,
    overrides: B,
    caption: C,
) -> impl Bundle {
    use bevy::feathers::constants::size;
    use bevy::feathers::focus::FocusIndicator;
    use bevy::feathers::font_styles::InheritableFont;
    use bevy::feathers::theme::{InheritableThemeTextColor, ThemeBackgroundColor};
    use bevy::feathers::tokens;
    use bevy::input_focus::tab_navigation::TabIndex;
    use bevy::picking::cursor::EntityCursor;
    use bevy::picking::hover::Hovered;
    use bevy::window::SystemCursorIcon;
    (
        Node {
            height: size::ROW_HEIGHT,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::horizontal(px(8)),
            border_radius: props.corners.to_border_radius(4.0),
            ..default()
        },
        bevy::ui_widgets::Button,
        props.variant,
        Hovered::default(),
        EntityCursor::System(SystemCursorIcon::Pointer),
        TabIndex(0),
        FocusIndicator,
        ThemeBackgroundColor(tokens::BUTTON_BG),
        InheritableThemeTextColor(tokens::BUTTON_TEXT),
        InheritableFont {
            font: Handle::default(),
            font_size: size::MEDIUM_FONT,
            weight: bevy::text::FontWeight::NORMAL,
        },
        overrides,
        Children::spawn(caption),
    )
}

/// Style overrides for text widgets. All fields are optional; `None` falls back to the theme default.
#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub font: Option<Handle<Font>>,
    pub font_size: Option<f32>,
    pub color: Option<Color>,
    pub justify: Option<bevy::text::Justify>,
    pub line_break: Option<bevy::text::LineBreak>,
}

impl TextStyle {
    /// Returns a `TextStyle` with only font size set.
    pub fn size(font_size: f32) -> Self {
        Self {
            font_size: Some(font_size),
            ..default()
        }
    }

    /// Returns a `TextStyle` with only color set.
    pub fn color(color: Color) -> Self {
        Self {
            color: Some(color),
            ..default()
        }
    }

    /// Returns a `TextStyle` with font size and color set.
    pub fn size_color(font_size: f32, color: Color) -> Self {
        Self {
            font_size: Some(font_size),
            color: Some(color),
            ..default()
        }
    }
}

/// Convenience helpers matching the palette module style.
pub const fn rect_all(val: Val) -> UiRect {
    UiRect::all(val)
}

/// A root UI node that fills the window and centers its content.
pub fn ui_root(name: impl Into<Cow<'static, str>>) -> impl Bundle {
    (
        Name::new(name),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20.0),
            ..default()
        },
        Pickable::IGNORE,
    )
}

/// A root UI node with a custom `Node` layout.
pub fn ui_root_with(name: impl Into<Cow<'static, str>>, node: Node) -> impl Bundle {
    (Name::new(name), node, Pickable::IGNORE)
}

/// A simple header label using the given theme.
pub fn header(text: impl Into<String>, theme: &TextTheme) -> impl Bundle {
    (
        Name::new("Header"),
        Text(text.into()),
        TextFont::default()
            .with_font(theme.font.clone())
            .with_font_size(theme.header_size),
        TextColor(theme.header_color),
    )
}

/// A simple text label using the given theme.
pub fn label(text: impl Into<String>, theme: &TextTheme) -> impl Bundle {
    (
        Name::new("Label"),
        Text(text.into()),
        TextFont::default()
            .with_font(theme.font.clone())
            .with_font_size(theme.label_size),
        TextColor(theme.label_color),
    )
}

/// A themed button with text and an action defined as an [`Observer`].
/// Uses the default button node layout (large, rounded, centered).
pub fn themed_button<M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
) -> impl Bundle
where
    I: IntoEntityObserver<M> + Sync,
{
    themed_button_with_node(
        text,
        action,
        theme,
        Node {
            width: px(380.0),
            height: px(80.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border_radius: theme.border_radius,
            ..default()
        },
    )
}

/// A small themed button with text and an action.
pub fn themed_button_small<M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
) -> impl Bundle
where
    I: IntoEntityObserver<M> + Sync,
{
    themed_button_with_node(
        text,
        action,
        theme,
        Node {
            width: px(30.0),
            height: px(30.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
    )
}

/// A themed button with a custom `Node` layout.
pub fn themed_button_with_node<M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
    button_node: Node,
) -> impl Bundle
where
    I: IntoEntityObserver<M> + Sync,
{
    let text = text.into();
    let bg = theme.bg;
    let palette = InteractionPalette {
        none: theme.bg,
        hovered: theme.bg_hovered,
        pressed: theme.bg_pressed,
    };
    let text_color = theme.text_color;
    let font = theme.font.clone();
    let font_size = theme.font_size;
    (
        Name::new("Button"),
        Node::default(),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent
                .spawn((
                    Name::new("Button Inner"),
                    // `bevy::ui_widgets::Button` (not the legacy `bevy_ui::widget::Button`):
                    // it emits `Activate` and tracks `Hovered`/`Pressed`, which is what
                    // `InteractionPalette` reads. The legacy button would instead bring an
                    // `Interaction` component that nothing in this crate looks at any more.
                    bevy::ui_widgets::Button,
                    Hovered::default(),
                    BackgroundColor(bg),
                    palette,
                    children![(
                        Name::new("Button Text"),
                        Text(text),
                        TextFont::default()
                            .with_font(font)
                            .with_font_size(font_size),
                        TextColor(text_color),
                        Pickable::IGNORE,
                    )],
                ))
                .insert(button_node)
                .observe(action);
        })),
    )
}

/// Spawn a horizontal progress bar with an optional [`ProgressBar`] component for dynamic updates.
///
/// `value` is the initial fill fraction (0.0–1.0).
pub fn progress_bar(
    value: f32,
    width: f32,
    height: f32,
    fill_color: Color,
    bg_color: Color,
) -> impl Bundle {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    (
        Name::new("ProgressBar"),
        ProgressBar { value, fill_color },
        Node {
            width: px(width),
            height: px(height),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(bg_color),
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            parent.spawn((
                Name::new("ProgressBarFill"),
                ProgressBarFill,
                Node {
                    width: percent(pct),
                    height: percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
            ));
        })),
    )
}

// ============================================================================
// Free-function helpers for raw ChildSpawner contexts
// ============================================================================

/// Spawn a selectable list-item row inside a `with_children` closure.
///
/// Use this inside `commands.entity(container).with_children(|parent| { ... })`
/// when you can't use the `UIBuilder` API. For UIBuilder contexts prefer
/// `ui.list_item(name, selected, handler)`.
pub fn spawn_list_item<M>(
    parent: &mut ChildSpawnerCommands<'_>,
    name: impl Into<String>,
    selected: bool,
    handler: impl IntoObserverSystem<Activate, M>,
) where
    M: 'static,
{
    let bg = if selected {
        Color::srgba(0.15, 0.40, 0.20, 0.95)
    } else {
        Color::srgba(0.07, 0.12, 0.09, 0.85)
    };
    let tc = if selected {
        Color::srgb(0.9, 1.0, 0.9)
    } else {
        Color::srgb(0.65, 0.80, 0.65)
    };
    parent
        .spawn((
            Node {
                width: percent(100.0),
                padding: UiRect::axes(px(6.0), px(4.0)),
                border_radius: BorderRadius::all(px(3.0)),
                ..default()
            },
            BackgroundColor(bg),
            InteractionPalette {
                none: bg,
                hovered: Color::srgba(0.20, 0.50, 0.28, 0.95),
                pressed: Color::srgba(0.12, 0.32, 0.18, 1.0),
            },
            bevy::picking::hover::Hovered::default(),
            bevy::ui_widgets::Button,
        ))
        .with_child((
            Text::new(name.into()),
            TextFont::default().with_font_size(11.0),
            TextColor(tc),
        ))
        .observe(handler);
}
