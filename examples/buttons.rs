//! Buttons showcase for `lava_ui_builder`, on the scene (BSN) API.
//!
//! Demonstrates:
//! - `scenes::button` with its action attached inline via `on(..)`
//! - variants (`Variant::Danger`, ...) and Bevy's state markers (`Selected`,
//!   `InteractionDisabled`) choosing the theme colours
//! - patching a widget: size overrides at the call site
//! - a status line bound to a resource instead of updated by a system
//! - `@FeathersButton`: bevy's own widget library composed into the same tree
//!
//! Run with: `cargo run --example buttons`

use bevy::feathers::theme::{ThemedText, UiTheme};
use bevy::feathers::{FeathersPlugins, controls::FeathersButton, dark_theme::create_dark_theme};
use bevy::prelude::*;
use lava_ui_builder::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FeathersPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        // Feathers widgets resolve their own tokens against their own theme resource.
        .insert_resource(UiTheme(create_dark_theme()))
        .init_resource::<Pressed>()
        .bind_resource::<Pressed>()
        .add_systems(Startup, scene.spawn())
        .run();
}

/// What the last button press was, so the demo can show that the observers fire.
#[derive(Resource, Default)]
struct Pressed(String);

fn scene() -> impl SceneList {
    bsn_list! { Camera2d -- @root() }
}

fn root() -> impl Scene {
    bsn! {
        @scenes::ui_root()
        Children [
            @scenes::header("Buttons")
            --
            @scenes::caption("") @scenes::bind_text(|p: &Pressed| format!("last pressed: {}", p.0))
            --
            @scenes::row(10.0)
            Children [
                @scenes::button("Primary") on(press("Primary"))
                --
                @scenes::button("Secondary") Variant::Secondary on(press("Secondary"))
                --
                @scenes::button("Success") Variant::Success on(press("Success"))
                --
                @scenes::button("Danger") Variant::Danger on(press("Danger"))
                --
                @scenes::button("Ghost") Variant::Ghost on(press("Ghost"))
            ]
            --
            @scenes::section_label("-- states: Bevy's own markers --")
            --
            @scenes::row(10.0)
            Children [
                @scenes::button("Selected") Selected on(press("Selected"))
                --
                @scenes::button("Disabled") InteractionDisabled
            ]
            --
            @scenes::section_label("-- patched sizes --")
            --
            @scenes::row(10.0)
            Node { align_items: AlignItems::Center }
            Children [
                @scenes::button("Big") Node { width: {px(240.0)}, height: {px(70.0)} } on(press("Big"))
                --
                @scenes::icon_button("X", 48.0) on(press("X"))
            ]
            --
            @scenes::section_label("-- bevy feathers --")
            --
            @scenes::row(8.0)
            Children [
                @FeathersButton
                on(press("Feathers default"))
                Children [ Text("Default") ThemedText ]
                --
                @FeathersButton
                InteractionDisabled
                Children [ Text("Disabled") ThemedText ]
            ]
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
