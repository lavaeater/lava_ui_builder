//! Game menu example for `lava_ui_builder` — Challenge 1, on the scene (BSN) API.
//!
//! Demonstrates sub-menu navigation driven by a `MenuScreen` resource.
//!   Main menu → Play / Audio Settings / Graphics Settings / Quit
//!   Audio panel → volume +/- buttons, mute toggle, Back
//!   Graphics panel → quality selector (Low/Medium/High), Back
//!
//! All three panels are spawned once and shown or hidden by `Node.display`; only the
//! text and the selection highlight are driven by systems.
//!
//! Run with: `cargo run --example game_menu`

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use lava_ui_builder::{scenes, InteractionPalette, LavaTheme, LavaUiPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .init_resource::<MenuScreen>()
        .init_resource::<AudioSettings>()
        .init_resource::<GraphicsSettings>()
        .add_systems(Startup, scene.spawn())
        .add_systems(
            Update,
            (sync_panels, sync_volume_text, sync_quality_buttons),
        )
        .run();
}

// ── Resources ─────────────────────────────────────────────────────────────────

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
enum MenuScreen {
    #[default]
    Main,
    Audio,
    Graphics,
}

#[derive(Resource)]
struct AudioSettings {
    volume: i32,
    muted: bool,
}
impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            volume: 7,
            muted: false,
        }
    }
}

#[derive(Default, PartialEq, Eq, Clone, Copy, Debug)]
enum GraphicsQuality {
    Low,
    #[default]
    Medium,
    High,
}

#[derive(Resource, Default)]
struct GraphicsSettings {
    quality: GraphicsQuality,
}

// ── Marker components ─────────────────────────────────────────────────────────
//
// Components used inside `bsn!` need `Default + Clone`.

#[derive(Component, Default, Clone)]
struct MainPanel;
#[derive(Component, Default, Clone)]
struct AudioPanel;
#[derive(Component, Default, Clone)]
struct GraphicsPanel;
#[derive(Component, Default, Clone)]
struct VolumeText;
#[derive(Component, Default, Clone)]
struct QualityButton(GraphicsQuality);

// ── Palette for the quality buttons ──────────────────────────────────────────
//
// Selection is a *palette* swap, not a `BackgroundColor` write: the palette system owns
// that component now, so anything writing it directly would be overwritten next frame.

const QUALITY_SELECTED: InteractionPalette = InteractionPalette {
    none: Color::srgb(0.20, 0.60, 0.30),
    hovered: Color::srgb(0.26, 0.70, 0.38),
    pressed: Color::srgb(0.16, 0.48, 0.24),
};
const QUALITY_UNSELECTED: InteractionPalette = InteractionPalette {
    none: Color::srgb(0.275, 0.400, 0.750),
    hovered: Color::srgb(0.384, 0.600, 0.820),
    pressed: Color::srgb(0.239, 0.286, 0.600),
};

// ── UI ────────────────────────────────────────────────────────────────────────

fn scene() -> impl SceneList {
    bsn_list![Camera2d, root()]
}

fn root() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        Children [
            (main_panel() MainPanel),
            (audio_panel() AudioPanel Node { display: Display::None }),
            (graphics_panel() GraphicsPanel Node { display: Display::None }),
        ]
    }
}

/// The shared panel chrome. Callers patch a marker and `display` onto it.
fn panel_shell() -> impl Scene {
    bsn! {
        scenes::column(16.0)
        Node {
            width: {px(340.0)},
            align_items: AlignItems::Center,
            padding: {UiRect::all(px(40.0))},
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(16.0))},
        }
        BackgroundColor(Color::srgba(0.08, 0.08, 0.14, 0.97))
        BorderColor::all(Color::srgb(0.25, 0.25, 0.5))
    }
}

/// A menu button: the standard themed button at a consistent size.
fn menu_button(label: &str, width: f32, height: f32) -> impl Scene {
    bsn! {
        scenes::button(label)
        Node { width: {px(width)}, height: {px(height)} }
    }
}

fn main_panel() -> impl Scene {
    bsn! {
        panel_shell()
        Children [
            scenes::text("MY GAME", 52.0, Color::WHITE),
            scenes::text("Main Menu", 18.0, Color::srgb(0.7, 0.7, 0.9)),
            (
                menu_button("Play", 260.0, 56.0)
                on(|_: On<Activate>| info!("Play!"))
            ),
            (
                menu_button("Audio", 260.0, 56.0)
                on(|_: On<Activate>, mut s: ResMut<MenuScreen>| *s = MenuScreen::Audio)
            ),
            (
                menu_button("Graphics", 260.0, 56.0)
                on(|_: On<Activate>, mut s: ResMut<MenuScreen>| *s = MenuScreen::Graphics)
            ),
            (
                scenes::button_colored(
                    "Quit",
                    Color::srgb(0.50, 0.10, 0.10),
                    Color::srgb(0.62, 0.14, 0.14),
                    Color::srgb(0.38, 0.08, 0.08),
                )
                Node { width: {px(260.0)}, height: {px(56.0)} }
                on(|_: On<Activate>, mut exit: MessageWriter<AppExit>| {
                    // Let bevy wind down cleanly instead of `process::exit`.
                    exit.write(AppExit::Success);
                })
            ),
        ]
    }
}

fn audio_panel() -> impl Scene {
    bsn! {
        panel_shell()
        Children [
            scenes::text("Audio Settings", 32.0, Color::WHITE),
            (
                scenes::row(12.0)
                Node { align_items: AlignItems::Center }
                Children [
                    scenes::text("Volume", 20.0, Color::srgb(0.8, 0.8, 0.8)),
                    (
                        menu_button("-", 40.0, 40.0)
                        on(|_: On<Activate>, mut s: ResMut<AudioSettings>| {
                            s.volume = s.volume.saturating_sub(1).max(0);
                        })
                    ),
                    (
                        scenes::text("7", 22.0, Color::WHITE)
                        Node { width: {px(36.0)} }
                        VolumeText
                    ),
                    (
                        menu_button("+", 40.0, 40.0)
                        on(|_: On<Activate>, mut s: ResMut<AudioSettings>| {
                            s.volume = s.volume.saturating_add(1).min(10);
                        })
                    ),
                ]
            ),
            (
                menu_button("Toggle Mute", 220.0, 48.0)
                on(|_: On<Activate>, mut s: ResMut<AudioSettings>| {
                    s.muted = !s.muted;
                    info!("Muted: {}", s.muted);
                })
            ),
            back_button(),
        ]
    }
}

fn graphics_panel() -> impl Scene {
    let choices: Vec<_> = [
        ("Low", GraphicsQuality::Low),
        ("Medium", GraphicsQuality::Medium),
        ("High", GraphicsQuality::High),
    ]
    .into_iter()
    .map(|(label, quality)| {
        bsn! {
            scenes::button_colored(
                label,
                QUALITY_UNSELECTED.none,
                QUALITY_UNSELECTED.hovered,
                QUALITY_UNSELECTED.pressed,
            )
            Node { width: {px(84.0)}, height: {px(48.0)} }
            QualityButton(quality)
            on(move |_: On<Activate>, mut s: ResMut<GraphicsSettings>| s.quality = quality)
        }
    })
    .collect();

    bsn! {
        panel_shell()
        Children [
            scenes::text("Graphics Settings", 32.0, Color::WHITE),
            scenes::text("Quality", 18.0, Color::srgb(0.7, 0.7, 0.9)),
            (
                scenes::row(8.0)
                Children [ {choices} ]
            ),
            back_button(),
        ]
    }
}

fn back_button() -> impl Scene {
    bsn! {
        scenes::button_colored(
            "Back",
            Color::srgb(0.20, 0.20, 0.35),
            Color::srgb(0.28, 0.28, 0.45),
            Color::srgb(0.15, 0.15, 0.28),
        )
        Node { width: {px(140.0)}, height: {px(44.0)} }
        on(|_: On<Activate>, mut s: ResMut<MenuScreen>| *s = MenuScreen::Main)
    }
}

// ── Update systems ────────────────────────────────────────────────────────────

#[allow(clippy::type_complexity)]
fn sync_panels(
    menu: Res<MenuScreen>,
    mut main_q: Query<&mut Node, (With<MainPanel>, Without<AudioPanel>, Without<GraphicsPanel>)>,
    mut audio_q: Query<&mut Node, (With<AudioPanel>, Without<MainPanel>, Without<GraphicsPanel>)>,
    mut graphics_q: Query<
        &mut Node,
        (With<GraphicsPanel>, Without<MainPanel>, Without<AudioPanel>),
    >,
) {
    if !menu.is_changed() {
        return;
    }
    let show = |node: &mut Node, visible: bool| {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    };
    if let Ok(mut n) = main_q.single_mut() {
        show(&mut n, *menu == MenuScreen::Main);
    }
    if let Ok(mut n) = audio_q.single_mut() {
        show(&mut n, *menu == MenuScreen::Audio);
    }
    if let Ok(mut n) = graphics_q.single_mut() {
        show(&mut n, *menu == MenuScreen::Graphics);
    }
}

fn sync_volume_text(audio: Res<AudioSettings>, mut texts: Query<&mut Text, With<VolumeText>>) {
    if !audio.is_changed() {
        return;
    }
    for mut t in &mut texts {
        **t = if audio.muted {
            "M".to_string()
        } else {
            audio.volume.to_string()
        };
    }
}

/// Selection swaps the whole palette rather than writing `BackgroundColor`: the palette
/// system owns that component, so a direct write would last exactly one frame.
fn sync_quality_buttons(
    gfx: Res<GraphicsSettings>,
    mut buttons: Query<(&QualityButton, &mut InteractionPalette)>,
) {
    if !gfx.is_changed() {
        return;
    }
    for (btn, mut palette) in &mut buttons {
        *palette = if btn.0 == gfx.quality {
            QUALITY_SELECTED
        } else {
            QUALITY_UNSELECTED
        };
    }
}
