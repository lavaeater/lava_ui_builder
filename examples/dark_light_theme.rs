//! Challenge 9: switching between themes at runtime.
//!
//! A main-menu-like screen with one of every kind of widget, and a cycle control that
//! picks the theme: dark, light, or one derived from a single accent colour. Every colour
//! on screen comes from the `LavaTheme` resource through tokens, so a switch is one
//! resource write and nothing is rebuilt. T steps the theme too.
//!
//! Run with: `cargo run --example dark_light_theme`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const THEMES: [&str; 4] = ["Dark", "Light", "Ember", "Ocean"];

#[derive(Resource, Default)]
struct ThemeChoice(usize);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .init_resource::<ThemeChoice>()
        .bind_resource::<ThemeChoice>()
        .add_systems(Startup, scene.spawn())
        .add_systems(Update, (apply_theme, step_with_t))
        .run();
}

fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @scenes::screen()
        Children [
            @scenes::card()
            Children [
                @scenes::title("Lava")
                --
                @scenes::caption("Every colour here is a theme token.")
                --
                @scenes::cycle(&THEMES, 0) AutoFocus
                @scenes::bind_choice(|t: &ThemeChoice| t.0, |t, i| t.0 = i)
                --
                @scenes::button("Play")
                --
                @scenes::row(10.0)
                Children [
                    @scenes::button("Options") Variant::Secondary
                    --
                    @scenes::button("Quit") Variant::Danger
                ]
                --
                @scenes::slider(0.0, 1.0, 0.6)
                --
                @scenes::toggle("Fullscreen", true)
                --
                @scenes::meter(0.7, 300.0, 10.0, ColorToken::HealthFill)
            ]
        ]
    }
}

/// One resource write per switch; every themed entity repaints next frame.
fn apply_theme(choice: Res<ThemeChoice>, mut theme: ResMut<LavaTheme>) {
    if !choice.is_changed() {
        return;
    }
    *theme = match choice.0 {
        1 => LavaTheme::light(),
        2 => LavaTheme::from_accent(Color::srgb(0.90, 0.38, 0.15)),
        3 => LavaTheme::from_accent(Color::srgb(0.15, 0.62, 0.78)),
        _ => LavaTheme::dark(),
    };
}

fn step_with_t(keys: Res<ButtonInput<KeyCode>>, mut choice: ResMut<ThemeChoice>) {
    if keys.just_pressed(KeyCode::KeyT) {
        choice.0 = (choice.0 + 1) % THEMES.len();
    }
}
