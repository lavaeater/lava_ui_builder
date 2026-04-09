//! HUD example for `lava_ui_builder` — Challenge 6.
//!
//! Demonstrates UI anchored to screen corners overlaid on a 2D scene.
//!   Top-left:     minimap placeholder
//!   Bottom-left:  HP counter + animated health bar
//!   Bottom-center: game time + team scores
//!   Bottom-right: ammo counter
//!
//! Run with: `cargo run --example hud`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_plugins(LavaUiPlugin)
        .insert_resource(GameState::default())
        .add_systems(Startup, (setup_camera, setup_scene, setup_hud))
        .add_systems(Update, (tick_game, sync_hp_bar, sync_hp_text, sync_score_text, sync_ammo_text))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Mock game state ───────────────────────────────────────────────────────────

#[derive(Resource)]
struct GameState {
    hp: f32,
    hp_max: f32,
    elapsed: f32,
    score_alpha: u32,
    score_bravo: u32,
    ammo: u32,
    ammo_max: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self { hp: 80.0, hp_max: 100.0, elapsed: 0.0, score_alpha: 7, score_bravo: 5, ammo: 23, ammo_max: 30 }
    }
}

fn tick_game(time: Res<Time>, mut state: ResMut<GameState>) {
    state.elapsed += time.delta_secs();
    // HP slowly drains as a demo
    state.hp = (state.hp - time.delta_secs() * 2.0).max(0.0);
}

// ── Scene ────────────────────────────────────────────────────────────────────

fn setup_scene(mut commands: Commands) {
    // Simple grid of colored quads as a fake game scene
    for x in -4..=4_i32 {
        for y in -3..=3_i32 {
            let hue = (x + y) as f32 * 0.08;
            commands.spawn((
                Sprite {
                    color: Color::hsl(hue * 360.0, 0.4, 0.25),
                    custom_size: Some(Vec2::splat(80.0)),
                    ..default()
                },
                Transform::from_xyz(x as f32 * 82.0, y as f32 * 82.0, 0.0),
            ));
        }
    }
}

// ── UI marker components ──────────────────────────────────────────────────────

#[derive(Component)] struct HpBar;
#[derive(Component)] struct HpText;
#[derive(Component)] struct ScoreText;
#[derive(Component)] struct AmmoText;

// ── HUD setup ─────────────────────────────────────────────────────────────────

fn setup_hud(commands: Commands, theme: Res<LavaTheme>, state: Res<GameState>) {
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    // Full-screen column: top row / flex-grow spacer / bottom row
    ui.set_node(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::SpaceBetween,
        padding: UiRect::all(Val::Px(16.0)),
        ..default()
    });

    // ── Top row ─────────────────────────────────────────────────────────
    ui.add_row(|top| {
        top.width_percent(100.0).justify_space_between().align_items_start();

        // Minimap (top-left)
        top.with_child(|mm| {
            mm.display_flex().flex_column()
                .align_items_center().justify_center()
                .size_px(160.0, 160.0)
                .bg_color(Color::srgba(0.05, 0.15, 0.05, 0.92))
                .border_all_px(2.0, Color::srgb(0.4, 0.6, 0.4))
                .border_radius_all_px(6.0);

            mm.add_text_child("[ MAP ]", Some(TextStyle::size_color(14.0, Color::srgb(0.5, 0.8, 0.5))));

            // Fake blips
            mm.add_row(|blips| {
                blips.gap_px(20.0).margin_top(Val::Px(10.0));
                for color in [Color::srgb(0.3, 0.6, 1.0), Color::srgb(1.0, 0.5, 0.3)] {
                    blips.with_child(|b| { b.size_px(8.0, 8.0).bg_color(color).border_radius_all_px(4.0); });
                }
            });
        });
    });

    // ── Bottom row ───────────────────────────────────────────────────────
    ui.add_row(|bottom| {
        bottom.width_percent(100.0).justify_space_between().align_items_end();

        // HP panel (bottom-left)
        bottom.with_child(|hp_panel| {
            hp_panel.display_flex().flex_column().gap_px(6.0)
                .padding_all_px(12.0)
                .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
                .border_all_px(1.0, Color::srgb(0.3, 0.3, 0.4))
                .border_radius_all_px(8.0);

            hp_panel.with_child(|label_row| {
                label_row.display_flex().flex_row().justify_space_between().width_px(200.0);
                label_row.add_text_child("HP", Some(TextStyle::size_color(14.0, Color::srgb(0.8, 0.8, 0.8))));
                label_row.with_child(|t| {
                    t.insert(HpText)
                        .with_text(
                            format!("{:.0}/{:.0}", state.hp, state.hp_max),
                            Some(TextStyle::size_color(14.0, Color::srgb(0.5, 1.0, 0.5))),
                        );
                });
            });

            hp_panel.with_child(|bar_wrap| {
                bar_wrap.insert(HpBar)
                    .insert_bundle(progress_bar(
                        state.hp / state.hp_max,
                        200.0, 16.0,
                        Color::srgb(0.2, 0.85, 0.3),
                        Color::srgb(0.15, 0.15, 0.15),
                    ));
            });
        });

        // Score panel (bottom-center)
        bottom.with_child(|score_panel| {
            score_panel.display_flex().flex_column()
                .align_items_center().gap_px(4.0)
                .padding_all_px(12.0)
                .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
                .border_all_px(1.0, Color::srgb(0.3, 0.3, 0.4))
                .border_radius_all_px(8.0);

            score_panel.add_text_child(
                game_time_str(state.elapsed),
                Some(TextStyle::size_color(14.0, Color::srgb(0.9, 0.9, 0.9))),
            );

            score_panel.with_child(|score_row| {
                score_row.display_flex().flex_row().gap_px(16.0).align_items_center();

                score_row.add_text_child("ALPHA", Some(TextStyle::size_color(14.0, Color::srgb(0.4, 0.6, 1.0))));
                score_row.with_child(|s| {
                    s.insert(ScoreText)
                        .with_text(
                            format!("{} — {}", state.score_alpha, state.score_bravo),
                            Some(TextStyle::size_color(22.0, Color::WHITE)),
                        );
                });
                score_row.add_text_child("BRAVO", Some(TextStyle::size_color(14.0, Color::srgb(1.0, 0.5, 0.3))));
            });
        });

        // Ammo panel (bottom-right)
        bottom.with_child(|ammo_panel| {
            ammo_panel.display_flex().flex_column()
                .align_items_end().gap_px(4.0)
                .padding_all_px(12.0)
                .bg_color(Color::srgba(0.0, 0.0, 0.0, 0.7))
                .border_all_px(1.0, Color::srgb(0.3, 0.3, 0.4))
                .border_radius_all_px(8.0);

            ammo_panel.add_text_child("AMMO", Some(TextStyle::size_color(12.0, Color::srgb(0.7, 0.7, 0.7))));

            ammo_panel.with_child(|t| {
                t.insert(AmmoText)
                    .with_text(
                        format!("{} / {}", state.ammo, state.ammo_max),
                        Some(TextStyle::size_color(28.0, Color::WHITE)),
                    );
            });

            // Ammo pip row
            ammo_panel.add_row(|pips| {
                pips.gap_px(3.0).margin_top(Val::Px(4.0));
                for i in 0..state.ammo_max {
                    let color = if i < state.ammo {
                        Color::srgb(1.0, 0.85, 0.2)
                    } else {
                        Color::srgb(0.2, 0.2, 0.2)
                    };
                    pips.with_child(|pip| {
                        pip.size_px(6.0, 16.0).bg_color(color).border_radius_all_px(2.0);
                    });
                }
            });
        });
    });

    ui.build();
}

// ── Runtime sync systems ──────────────────────────────────────────────────────

fn sync_hp_bar(state: Res<GameState>, mut bars: Query<&mut ProgressBar, With<HpBar>>) {
    if !state.is_changed() { return; }
    for mut bar in &mut bars {
        bar.value = state.hp / state.hp_max;
    }
}

fn sync_hp_text(state: Res<GameState>, mut texts: Query<&mut Text, With<HpText>>) {
    if !state.is_changed() { return; }
    for mut t in &mut texts {
        **t = format!("{:.0}/{:.0}", state.hp, state.hp_max);
    }
}

fn sync_score_text(state: Res<GameState>, mut texts: Query<&mut Text, With<ScoreText>>) {
    if !state.is_changed() { return; }
    for mut t in &mut texts {
        **t = format!("{} — {}", state.score_alpha, state.score_bravo);
    }
}

fn sync_ammo_text(state: Res<GameState>, mut texts: Query<&mut Text, With<AmmoText>>) {
    if !state.is_changed() { return; }
    for mut t in &mut texts {
        **t = format!("{} / {}", state.ammo, state.ammo_max);
    }
}

fn game_time_str(elapsed: f32) -> String {
    let mins = (elapsed as u32) / 60;
    let secs = (elapsed as u32) % 60;
    format!("{:02}:{:02}", mins, secs)
}
