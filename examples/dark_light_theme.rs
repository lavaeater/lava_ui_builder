//! Dark/Light Theme example for `lava_ui_builder` — Challenge 9, on the scene (BSN) API.
//!
//! This is the example theme tokens exist for. The previous version rebuilt the entire
//! UI on every switch (`start_from_entity(.., clear_children: true)` plus a full
//! repopulate). Here the tree is spawned exactly once at startup: switching themes writes
//! the `LavaTheme` resource and `tokens::apply_theme_tokens` repaints the live entities.
//!
//! Nothing is despawned, so entity ids, observers and any state held on those entities
//! all survive the switch — watch the click counter keep counting across it.
//!
//! Run with: `cargo run --example dark_light_theme`

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{
    scenes, ButtonTheme, ColorToken, FontToken, LavaTheme, LavaUiPlugin, TextTheme,
    ThemedBackground, ThemedBorderColor, ThemedFont,
};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(dark_theme())
        .init_resource::<ThemeMode>()
        .init_resource::<Clicks>()
        .add_systems(Startup, scene.spawn())
        .add_systems(
            Update,
            (
                apply_theme_mode.run_if(resource_changed::<ThemeMode>),
                update_labels,
            ),
        )
        .run();
}

// ── Theme presets ─────────────────────────────────────────────────────────────

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum ThemeMode {
    #[default]
    Dark,
    Light,
}

#[derive(Resource, Default)]
struct Clicks(u32);

fn dark_theme() -> LavaTheme {
    LavaTheme::default()
}

fn light_theme() -> LavaTheme {
    LavaTheme {
        button: ButtonTheme {
            bg: Color::srgb(0.72, 0.72, 0.88),
            bg_hovered: Color::srgb(0.60, 0.60, 0.76),
            bg_pressed: Color::srgb(0.50, 0.50, 0.66),
            text_color: Color::srgb(0.10, 0.10, 0.15),
            border_color: Color::srgb(0.50, 0.50, 0.65),
            collapsible_bg: Color::srgb(0.75, 0.75, 0.82),
            collapsible_bg_hovered: Color::srgb(0.65, 0.65, 0.72),
            collapsible_bg_pressed: Color::srgb(0.55, 0.70, 0.55),
            ..ButtonTheme::default()
        },
        text: TextTheme {
            header_color: Color::srgb(0.10, 0.10, 0.18),
            label_color: Color::srgb(0.22, 0.22, 0.30),
            ..TextTheme::default()
        },
        bg_color: Color::srgb(0.88, 0.88, 0.94),
        border_color: Color::srgb(0.55, 0.55, 0.68),
        ..LavaTheme::default()
    }
}

// ── Markers for the two labels that change text (not colour) ─────────────────

#[derive(Component, Default, Clone)]
struct ToggleLabel;

#[derive(Component, Default, Clone)]
struct ClickLabel;

// ── UI, spawned once ─────────────────────────────────────────────────────────

fn scene() -> impl SceneList {
    bsn_list![Camera2d, root()]
}

fn root() -> impl Scene {
    let options: Vec<_> = ["Option A", "Option B", "Option C"]
        .into_iter()
        .map(|label| {
            bsn! {
                scenes::button(label)
                Node { width: percent(100), height: {px(44.0)} }
                on(count_click)
            }
        })
        .collect();

    bsn! {
        scenes::ui_root()
        // The root's own background is a token too, so the whole window repaints.
        ThemedBackground(ColorToken::PanelBg)
        Children [
            scenes::header("Theme Showcase"),
            (
                scenes::column(12.0)
                Node {
                    width: {px(420.0)},
                    padding: {UiRect::all(px(24.0))},
                    border: {UiRect::all(px(2.0))},
                    border_radius: {BorderRadius::all(px(12.0))},
                }
                ThemedBackground(ColorToken::PanelBg)
                ThemedBorderColor(ColorToken::PanelBorder)
                Children [
                    scenes::header("Sample Panel") ThemedFont(FontToken::Label),
                    scenes::label("Every colour here is a token. Switching the theme\nrepaints these entities in place."),
                    (scenes::label("clicks: 0") ClickLabel),
                    {options},
                ]
            ),
            (
                scenes::button("Switch to Light")
                Node { width: {px(240.0)}, height: {px(52.0)} }
                ToggleLabel
                on(|_: On<Activate>, mut mode: ResMut<ThemeMode>| {
                    *mode = match *mode {
                        ThemeMode::Dark => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                    };
                })
            ),
        ]
    }
}

fn count_click(_activate: On<Activate>, mut clicks: ResMut<Clicks>) {
    clicks.0 += 1;
}

// ── The entire theme switch ──────────────────────────────────────────────────

/// One resource write. There is no UI code here at all: `apply_theme_tokens` does the
/// rest, next frame, for every entity carrying a token.
fn apply_theme_mode(mode: Res<ThemeMode>, mut theme: ResMut<LavaTheme>) {
    *theme = match *mode {
        ThemeMode::Dark => dark_theme(),
        ThemeMode::Light => light_theme(),
    };
}

/// Text content is not a colour, so it is not a token -- these two labels are updated the
/// ordinary way, by a system with a marker query.
fn update_labels(
    mode: Res<ThemeMode>,
    clicks: Res<Clicks>,
    toggle: Single<&Children, With<ToggleLabel>>,
    click_label: Single<&mut Text, (With<ClickLabel>, Without<ToggleLabel>)>,
    mut texts: Query<&mut Text, Without<ClickLabel>>,
) {
    if clicks.is_changed() {
        let mut label = click_label;
        ***label = format!("clicks: {}", clicks.0);
    }
    if mode.is_changed() {
        // The button's caption lives on its child entity.
        for child in toggle.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = match *mode {
                    ThemeMode::Dark => "Switch to Light".to_string(),
                    ThemeMode::Light => "Switch to Dark".to_string(),
                };
            }
        }
    }
}
