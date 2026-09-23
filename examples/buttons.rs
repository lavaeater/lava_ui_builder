//! Buttons showcase for `lava_ui_builder`, on the scene (BSN) API.
//!
//! Demonstrates:
//! - `scenes::button` with its action attached inline via `on(..)`
//! - patching a widget: size and layout overrides at the call site
//! - `scenes::button_colored` for a button whose colour is meaning, not theme
//! - marker components on scene entities, driven by ordinary systems
//! - `@FeathersButton`: bevy's own widget library composed into the same tree
//!
//! Run with: `cargo run --example buttons`

use bevy::feathers::theme::{ThemedText, UiTheme};
use bevy::feathers::{controls::FeathersButton, dark_theme::create_dark_theme, FeathersPlugins};
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{scenes, LavaTheme, LavaUiPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        // Feathers widgets resolve their own tokens against their own theme resource.
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(Pressed::default())
        .add_systems(Startup, scene.spawn())
        .add_systems(Update, report.run_if(resource_changed::<Pressed>))
        .run();
}

/// What the last button press was, so the demo can show that the observers fire.
#[derive(Resource, Default)]
struct Pressed(String);

#[derive(Component, Default, Clone)]
struct StatusLine;

fn scene() -> impl SceneList {
    bsn_list![Camera2d, root()]
}

fn root() -> impl Scene {
    bsn! {
        scenes::ui_root()
        Children [
            scenes::header("Buttons"),
            (scenes::text("", 20.0, Color::srgb(0.7, 0.9, 0.7)) StatusLine),

            scenes::section_label("-- themed --"),
            (
                scenes::button("Play")
                on(press("Play"))
            ),
            (
                // Patching: same widget, bigger. Node is layout, so it patches cleanly.
                scenes::button("Settings")
                Node { width: {px(240.0)}, height: {px(70.0)} }
                on(press("Settings"))
            ),
            (
                scenes::button("X")
                Node { width: {px(48.0)}, height: {px(48.0)} }
                on(press("X"))
            ),

            scenes::section_label("-- explicit colour --"),
            (
                // Not themed: a red Quit stays red through a theme switch.
                scenes::button_colored(
                    "Quit",
                    Color::srgb(0.60, 0.15, 0.15),
                    Color::srgb(0.75, 0.20, 0.20),
                    Color::srgb(0.45, 0.10, 0.10),
                )
                on(press("Quit"))
            ),

            scenes::section_label("-- bevy feathers --"),
            (
                scenes::row(8.0)
                Children [
                    (
                        @FeathersButton
                        on(press("Feathers default"))
                        Children [( Text("Default") ThemedText )]
                    ),
                    (
                        @FeathersButton
                        InteractionDisabled
                        Children [( Text("Disabled") ThemedText )]
                    ),
                ]
            ),
        ]
    }
}

/// One observer factory for every button: `on(..)` needs a `Clone` system, and a closure
/// that captures only a `&'static str` is `Clone`.
fn press(what: &'static str) -> impl Fn(On<Activate>, ResMut<Pressed>) + Clone {
    move |_activate: On<Activate>, mut pressed: ResMut<Pressed>| {
        pressed.0 = what.to_string();
    }
}

fn report(pressed: Res<Pressed>, mut status: Single<&mut Text, With<StatusLine>>) {
    info!("{} pressed", pressed.0);
    ***status = format!("last pressed: {}", pressed.0);
}
