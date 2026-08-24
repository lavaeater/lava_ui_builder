//! Game menu example for `lava_ui_builder` — Challenge 1.
//!
//! Demonstrates sub-menu navigation driven by a `MenuScreen` resource.
//!   Main menu → Play / Audio Settings / Graphics Settings / Quit
//!   Audio panel → volume +/- buttons, mute toggle, Back
//!   Graphics panel → quality selector (Low/Medium/High), Back
//!
//! Run with: `cargo run --example game_menu`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_plugins(LavaUiPlugin)
        .init_resource::<MenuScreen>()
        .init_resource::<AudioSettings>()
        .init_resource::<GraphicsSettings>()
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(
            Update,
            (sync_panels, sync_volume_text, sync_quality_buttons),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
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

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
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

#[derive(Component)]
struct MainPanel;
#[derive(Component)]
struct AudioPanel;
#[derive(Component)]
struct GraphicsPanel;
#[derive(Component)]
struct VolumeText;
#[derive(Component)]
struct QualityButton(GraphicsQuality);

// ── UI build ──────────────────────────────────────────────────────────────────

fn setup_ui(commands: Commands, theme: Res<LavaTheme>) {
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.set_node(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        ..default()
    });

    build_main_panel(&mut ui, &theme);
    build_audio_panel(&mut ui, &theme);
    build_graphics_panel(&mut ui, &theme);

    ui.build();
}

fn panel_shell(ui: &mut UIBuilder, f: impl FnOnce(&mut UIBuilder)) {
    ui.display_flex()
        .flex_column()
        .align_items_center()
        .gap_px(16.0)
        .padding_all_px(40.0)
        .bg_color(Color::srgba(0.08, 0.08, 0.14, 0.97))
        .border_all_px(2.0, Color::srgb(0.25, 0.25, 0.5))
        .border_radius_all_px(16.0)
        .width_px(340.0);
    f(ui);
}

fn build_main_panel(ui: &mut UIBuilder, _theme: &LavaTheme) {
    ui.with_child(|panel| {
        panel.insert(MainPanel);
        panel_shell(panel, |p| {
            p.add_text_child("MY GAME", Some(TextStyle::size_color(52.0, Color::WHITE)));
            p.add_text_child(
                "Main Menu",
                Some(TextStyle::size_color(18.0, Color::srgb(0.7, 0.7, 0.9))),
            );

            p.add_button_observe(
                "  Play  ",
                |btn| {
                    btn.size_px(260.0, 56.0);
                },
                |_: On<Activate>| info!("▶ Play!"),
            );

            p.add_button_observe(
                "  Audio  ",
                |btn| {
                    btn.size_px(260.0, 56.0);
                },
                |_: On<Activate>, mut s: ResMut<MenuScreen>| {
                    *s = MenuScreen::Audio;
                },
            );

            p.add_button_observe(
                "  Graphics  ",
                |btn| {
                    btn.size_px(260.0, 56.0);
                },
                |_: On<Activate>, mut s: ResMut<MenuScreen>| {
                    *s = MenuScreen::Graphics;
                },
            );

            p.add_button_observe(
                "  Quit  ",
                |btn| {
                    btn.size_px(260.0, 56.0)
                        .bg_color(Color::srgb(0.5, 0.1, 0.1));
                },
                |_: On<Activate>| {
                    std::process::exit(0);
                },
            );
        });
    });
}

fn build_audio_panel(ui: &mut UIBuilder, _theme: &LavaTheme) {
    ui.with_child(|panel| {
        panel.insert(AudioPanel).display_none();
        panel_shell(panel, |p| {
            p.add_text_child(
                "Audio Settings",
                Some(TextStyle::size_color(32.0, Color::WHITE)),
            );

            // Volume row: label  [ - ]  value  [ + ]
            p.add_row(|row| {
                row.gap_px(12.0).align_items_center();

                row.add_text_child(
                    "Volume",
                    Some(TextStyle::size_color(20.0, Color::srgb(0.8, 0.8, 0.8))),
                );

                row.add_button_observe(
                    " − ",
                    |btn| {
                        btn.size_px(40.0, 40.0);
                    },
                    |_: On<Activate>, mut s: ResMut<AudioSettings>| {
                        s.volume = (s.volume - 1).max(0);
                    },
                );

                row.with_child(|t| {
                    t.insert(VolumeText)
                        .width_px(36.0)
                        .with_text("7", Some(TextStyle::size_color(22.0, Color::WHITE)));
                });

                row.add_button_observe(
                    " + ",
                    |btn| {
                        btn.size_px(40.0, 40.0);
                    },
                    |_: On<Activate>, mut s: ResMut<AudioSettings>| {
                        s.volume = (s.volume + 1).min(10);
                    },
                );
            });

            // Mute toggle
            p.add_button_observe(
                "  Toggle Mute  ",
                |btn| {
                    btn.size_px(220.0, 48.0);
                },
                |_: On<Activate>, mut s: ResMut<AudioSettings>| {
                    s.muted = !s.muted;
                    info!("Muted: {}", s.muted);
                },
            );

            back_button(p);
        });
    });
}

fn build_graphics_panel(ui: &mut UIBuilder, _theme: &LavaTheme) {
    ui.with_child(|panel| {
        panel.insert(GraphicsPanel).display_none();
        panel_shell(panel, |p| {
            p.add_text_child(
                "Graphics Settings",
                Some(TextStyle::size_color(32.0, Color::WHITE)),
            );
            p.add_text_child(
                "Quality",
                Some(TextStyle::size_color(18.0, Color::srgb(0.7, 0.7, 0.9))),
            );

            p.add_row(|row| {
                row.gap_px(8.0);

                for (label, quality) in [
                    ("Low", GraphicsQuality::Low),
                    ("Medium", GraphicsQuality::Medium),
                    ("High", GraphicsQuality::High),
                ] {
                    row.add_button_observe(
                        label,
                        move |btn| {
                            btn.size_px(84.0, 48.0).insert(QualityButton(quality));
                        },
                        move |_: On<Activate>, mut s: ResMut<GraphicsSettings>| {
                            s.quality = quality;
                        },
                    );
                }
            });

            back_button(p);
        });
    });
}

fn back_button(p: &mut UIBuilder) {
    p.add_button_observe(
        "← Back",
        |btn| {
            btn.size_px(140.0, 44.0)
                .bg_color(Color::srgb(0.2, 0.2, 0.35));
        },
        |_: On<Activate>, mut s: ResMut<MenuScreen>| {
            *s = MenuScreen::Main;
        },
    );
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
    let show = |display: &mut Node, visible: bool| {
        display.display = if visible {
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

fn sync_quality_buttons(
    gfx: Res<GraphicsSettings>,
    mut buttons: Query<(&QualityButton, &mut BackgroundColor)>,
) {
    if !gfx.is_changed() {
        return;
    }
    for (btn, mut bg) in &mut buttons {
        bg.0 = if btn.0 == gfx.quality {
            Color::srgb(0.2, 0.6, 0.3)
        } else {
            Color::srgb(0.275, 0.400, 0.750)
        };
    }
}
