//! Challenge 1: a game menu with audio and graphics sub-menus.
//!
//! - Screens are `States`: `add_screen` spawns each on enter and despawns it on exit;
//!   `go_to` buttons switch, Escape / gamepad B / the Back button go back.
//! - The slider, toggle and dropdown are bound to the `Audio` / `Graphics` resources in
//!   both directions, so no system copies values between UI and game.
//! - Mouse: hover and press styles. Keyboard (Tab, arrows, Enter) and controller (D-pad /
//!   stick, A, B): a focus ring on the focused widget.
//!
//! Run with: `cargo run --example game_menu`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Menu {
    #[default]
    Main,
    Audio,
    Graphics,
}

#[derive(Resource)]
struct Audio {
    volume: f32,
    muted: bool,
}

#[derive(Resource, Default)]
struct Graphics {
    quality: usize,
}

const QUALITIES: [&str; 3] = ["Low", "Medium", "High"];

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .insert_resource(Audio { volume: 0.7, muted: false })
        .insert_resource(Graphics { quality: 1 })
        .bind_resource::<Audio>()
        .bind_resource::<Graphics>()
        .init_state::<Menu>()
        .add_screen(Menu::Main, main_menu)
        .add_screen(Menu::Audio, audio_menu)
        .add_screen(Menu::Graphics, graphics_menu)
        .screen_back(Menu::Audio, Menu::Main)
        .screen_back(Menu::Graphics, Menu::Main)
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .run();
}

/// Every menu screen: a card on a centred root.
fn menu_card(title: &str, contents: impl SceneList) -> impl Scene {
    let title = title.to_string();
    bsn! {
        @scenes::screen()
        Children [
            @scenes::card()
            Children [ @scenes::header(title) -- {contents} ]
        ]
    }
}

fn main_menu() -> impl Scene {
    menu_card("My Game", bsn_list! {
        @scenes::button("Play") AutoFocus
        on(|_: On<Activate>, mut commands: Commands| commands.trigger(Toast::info("Starting...")))
        --
        @scenes::button("Audio") Variant::Secondary @scenes::go_to(Menu::Audio)
        --
        @scenes::button("Graphics") Variant::Secondary @scenes::go_to(Menu::Graphics)
        --
        @scenes::button("Quit") Variant::Danger
        on(|_: On<Activate>, mut commands: Commands| {
            commands
                .spawn_scene(scenes::confirm("Quit?", "Unsaved progress will be lost.", "Quit"))
                .observe(|answer: On<DialogResult>, mut exit: MessageWriter<AppExit>| {
                    if answer.confirmed {
                        exit.write(AppExit::Success);
                    }
                });
        })
    })
}

fn audio_menu() -> impl Scene {
    menu_card("Audio", bsn_list! {
        @scenes::caption("Volume")
        --
        @scenes::slider(0.0, 1.0, 0.7) AutoFocus
        @scenes::bind_slider(|a: &Audio| a.volume, |a, v| a.volume = v)
        --
        @scenes::label("") @scenes::bind_text(|a: &Audio| format!("{:.0}%", a.volume * 100.0))
        --
        @scenes::toggle("Mute", false)
        @scenes::bind_checked(|a: &Audio| a.muted, |a, on| a.muted = on)
        --
        @scenes::button("Back") Variant::Ghost Back
    })
}

fn graphics_menu() -> impl Scene {
    menu_card("Graphics", bsn_list! {
        @scenes::caption("Quality")
        --
        @scenes::select(&QUALITIES, 1) @scenes::tooltip("Higher looks better, runs slower")
        @scenes::bind_choice(|g: &Graphics| g.quality, |g, i| g.quality = i)
        --
        @scenes::button("Back") Variant::Ghost Back
    })
}
