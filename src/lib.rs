use std::borrow::Cow;
use std::collections::VecDeque;

use bevy::color::palettes::basic::WHITE;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

mod builder;
mod button_builder;
pub mod systems;

// Re-export feathers types for external use
pub use bevy::feathers::controls::{button as feathers_button_fn, ButtonProps, ButtonVariant};
pub use bevy::feathers::rounded_corners::RoundedCorners;
pub use bevy::feathers::theme::ThemedText;
pub use bevy::ui::InteractionDisabled;
pub use bevy::ui_widgets::Activate;
use bevy::window::{WindowResized};
// ============================================================================
// Theme types — slim replacements for the old NodeDef/NodePartial/ButtonDef/etc.
// ============================================================================

/// Theme-level colors and fonts for buttons.
#[derive(Clone, Debug)]
pub struct ButtonTheme {
    pub bg: Color,
    pub bg_hovered: Color,
    pub bg_pressed: Color,
    pub text_color: Color,
    pub font: Handle<Font>,
    pub font_size: f32,
    pub border_radius: BorderRadius,
    pub border_width: Val,
    pub border_color: Color,
    pub height: Val,
    pub width: Val,
    /// Colors for collapsible section toggle buttons.
    pub collapsible_bg: Color,
    pub collapsible_bg_hovered: Color,
    pub collapsible_bg_pressed: Color,
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self {
            bg: Color::srgb(0.275, 0.400, 0.750),
            bg_hovered: Color::srgb(0.384, 0.600, 0.820),
            bg_pressed: Color::srgb(0.239, 0.286, 0.600),
            text_color: Color::from(WHITE),
            font: Default::default(),
            font_size: 40.0,
            border_radius: BorderRadius::MAX,
            border_width: px(2.0),
            border_color: Color::srgb(0.2, 0.2, 0.2),
            height: px(50.0),
            width: px(150.0),
            collapsible_bg: Color::srgb(0.25, 0.25, 0.3),
            collapsible_bg_hovered: Color::srgb(0.35, 0.35, 0.4),
            collapsible_bg_pressed: Color::srgb(0.4, 0.6, 0.4),
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
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            font: Default::default(),
            header_size: 40.0,
            label_size: 24.0,
            header_color: Color::srgb(0.988, 0.984, 0.800),
            label_color: Color::srgb(0.867, 0.827, 0.412),
        }
    }
}

/// Top-level UI theme resource. Insert this as a `Resource` to configure the look of
/// bundle-function widgets and the imperative `UIBuilder`.
#[derive(Resource, Clone, Debug)]
pub struct LavaTheme {
    pub button: ButtonTheme,
    pub text: TextTheme,
    pub bg_color: Color,
    pub border_color: Color,
    pub ui_width: f32,
    pub ui_height: f32,
}

impl Default for LavaTheme {
    fn default() -> Self {
        Self {
            button: ButtonTheme::default(),
            text: TextTheme::default(),
            bg_color: Color::srgba(0.5, 0.5, 0.5, 0.25),
            border_color: Color::srgba(0.2, 0.2, 0.2, 0.25),
            ui_width: 1920.0,
            ui_height: 1080.0,
        }
    }
}

/// Interaction palette for hover/press color changes.
#[derive(Component, Clone, Debug)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

// ============================================================================
// Bundle-function widgets — inspired by the widget.rs pattern.
// These return `impl Bundle` and compose with `children![]`.
// ============================================================================

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
        Self { font_size: Some(font_size), ..default() }
    }

    /// Returns a `TextStyle` with only color set.
    pub fn color(color: Color) -> Self {
        Self { color: Some(color), ..default() }
    }

    /// Returns a `TextStyle` with font size and color set.
    pub fn size_color(font_size: f32, color: Color) -> Self {
        Self { font_size: Some(font_size), color: Some(color), ..default() }
    }
}

/// Convenience helpers matching the palette module style.
// pub fn px(val: f32) -> Val {
//     Val::Px(val)
// }
// 
// pub fn percent(val: f32) -> Val {
//     Val::Percent(val)
// }

pub fn rect_all(val: Val) -> UiRect {
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
pub fn themed_button<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
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
pub fn themed_button_small<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
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
pub fn themed_button_with_node<E, B, M, I>(
    text: impl Into<String>,
    action: I,
    theme: &ButtonTheme,
    button_node: Node,
) -> impl Bundle
where
    E: EntityEvent,
    B: Bundle,
    I: IntoObserverSystem<E, B, M>,
{
    let text = text.into();
    let action = IntoObserverSystem::into_system(action);
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
                    Button,
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

// ============================================================================
// Imperative UIBuilder — kept for dynamic/runtime UI construction.
// ============================================================================

/// Fluent UI Builder for creating Bevy UI elements imperatively.
/// Best suited for dynamic, data-driven, or runtime-rebuilt UI trees.
/// For static menus, prefer the bundle-function widgets above with `children![]`.
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
// Collapsible widget components
// ============================================================================

/// Marker component for a collapsible UI section.
/// When collapsed, the content is hidden and only the toggle button is shown.
#[derive(Component)]
pub struct Collapsible {
    pub collapsed: bool,
    pub label: String,
}

impl Collapsible {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            collapsed: false,
            label: label.into(),
        }
    }

    pub fn collapsed(label: impl Into<String>) -> Self {
        Self {
            collapsed: true,
            label: label.into(),
        }
    }
}

/// Marker for the toggle button that collapses/expands a Collapsible section.
#[derive(Component)]
pub struct CollapseToggleButton {
    pub target: Entity,
}

/// Marker for the content container inside a Collapsible section.
#[derive(Component)]
pub struct CollapsibleContent {
    pub parent: Entity,
}

// ============================================================================
// Progress bar widget
// ============================================================================

/// Component that drives the fill percentage of a [`progress_bar`] widget.
/// Change `value` (0.0–1.0) and the `sync_progress_bars` system will update the inner fill node.
#[derive(Component, Clone, Debug)]
pub struct ProgressBar {
    /// Fill fraction, clamped to `0.0..=1.0`.
    pub value: f32,
    pub fill_color: Color,
}

/// Marker placed on the inner fill node spawned by [`progress_bar`].
#[derive(Component)]
pub struct ProgressBarFill;

/// Spawn a horizontal progress bar with an optional [`ProgressBar`] component for dynamic updates.
///
/// `value` is the initial fill fraction (0.0–1.0).
pub fn progress_bar(value: f32, width: f32, height: f32, fill_color: Color, bg_color: Color) -> impl Bundle {
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
// Plugin
// ============================================================================

/// Plugin that registers generic UI systems for collapsible sections,
/// scroll handling, and interaction palette color changes.
/// Add this plugin to your app to get these systems automatically.
pub struct LavaUiPlugin;

impl Plugin for LavaUiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiScale(1.0)).add_systems(
            Update,
            (
                systems::handle_scroll_input,
                systems::handle_collapse_toggle,
                systems::update_collapsible_visibility,
                systems::apply_interaction_palette,
                systems::sync_progress_bars,
                adapt_ui_scale,
            ),
        );
    }
}

fn adapt_ui_scale(
    mut message_reader: MessageReader<WindowResized>,
    mut ui_scale: ResMut<UiScale>,
    theme: Option<Res<LavaTheme>>,
) {
    for event in message_reader.read() {
        let base_width = theme.as_ref().map(|t| t.ui_width).unwrap_or(1920.0);
        ui_scale.0 = (event.width / base_width).max(0.5);
        info!("Ui scale: {}", ui_scale.0);
    }
}
