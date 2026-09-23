//! The scene (BSN) API, side by side with what it replaces.
//!
//! This is `basic_layout` rebuilt with `bsn!` instead of the bundle functions and the
//! imperative `UIBuilder`. Compare the two files: the tree here is one expression, the
//! observers sit next to the buttons they belong to, and per-widget overrides are
//! patches rather than option arguments.
//!
//! Run with: `cargo run --example bsn_layout`

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{ColorToken, ThemedTextColor};
use lava_ui_builder::{scenes, ButtonTheme, LavaTheme, LavaUiPlugin, TextTheme};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LavaUiPlugin)
        .insert_resource(LavaTheme::default())
        .insert_resource(Clicks(0))
        .add_systems(Startup, scene.spawn())
        .add_systems(
            Update,
            (
                sync_click_label.run_if(resource_changed::<Clicks>),
                toggle_theme_on_key,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Clicks(u32);

/// Marker so an ordinary system can find the text to update. Scene-spawned entities are
/// ordinary entities; nothing about querying them changes.
#[derive(Component, Default, Clone)]
struct ClickLabel;

fn scene() -> impl SceneList {
    bsn_list![Camera2d, root()]
}

fn root() -> impl Scene {
    // A runtime-built list of children: `Vec<impl Scene>` is a `SceneList`, so it
    // splices straight into `Children [ .. ]`. This is what `foreach_child` was for.
    let rows: Vec<_> = ["alpha", "bravo", "charlie"]
        .into_iter()
        .map(scenes::label)
        .collect();

    // Conditional children are `Option<impl SceneList>` -- note *List*: in a
    // `Children [ .. ]` position an interpolated value has to be a list, so the branch
    // is wrapped in `bsn_list!` rather than being a bare scene.
    let show_hint = true;
    let hint = show_hint.then(|| bsn_list![scenes::label("press T to switch theme")]);

    bsn! {
        #Root
        scenes::ui_root()
        Children [
            scenes::header("Lava UI Builder"),
            {hint},
            (
                // Same widget, a different token patched over it: still theme-driven,
                // so it follows a theme switch too.
                scenes::label("patched to the header token")
                ThemedTextColor(ColorToken::HeaderText)
            ),
            (
                scenes::button("Click me")
                on(|_: On<Activate>, mut clicks: ResMut<Clicks>| clicks.0 += 1)
            ),
            (
                scenes::label("clicks: 0")
                ClickLabel
            ),
            (
                Node { flex_direction: FlexDirection::Column, row_gap: px(4) }
                Children [ {rows} ]
            ),
            scenes::progress_bar(0.45, 240.0, 16.0,
                Color::srgb(0.35, 0.75, 0.45), Color::srgb(0.15, 0.15, 0.18)),
            scenes::collapsible("Details", false, bsn_list![
                scenes::label("hidden when collapsed"),
                scenes::label("click the header to toggle"),
            ]),
        ]
    }
}

fn sync_click_label(clicks: Res<Clicks>, mut label: Single<&mut Text, With<ClickLabel>>) {
    ***label = format!("clicks: {}", clicks.0);
}

/// The payoff of theme tokens: swapping the resource repaints the live UI. Nothing is
/// despawned, no scene is respawned, and the click count survives the switch.
fn toggle_theme_on_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut theme: ResMut<LavaTheme>,
    mut light: Local<bool>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    *light = !*light;
    *theme = if *light { light_theme() } else { LavaTheme::default() };
}

fn light_theme() -> LavaTheme {
    LavaTheme {
        button: ButtonTheme {
            bg: Color::srgb(0.72, 0.72, 0.88),
            bg_hovered: Color::srgb(0.60, 0.60, 0.76),
            bg_pressed: Color::srgb(0.50, 0.50, 0.66),
            text_color: Color::srgb(0.10, 0.10, 0.15),
            collapsible_bg: Color::srgb(0.75, 0.75, 0.82),
            ..ButtonTheme::default()
        },
        text: TextTheme {
            header_color: Color::srgb(0.10, 0.10, 0.18),
            label_color: Color::srgb(0.22, 0.22, 0.30),
            ..TextTheme::default()
        },
        ..LavaTheme::default()
    }
}
