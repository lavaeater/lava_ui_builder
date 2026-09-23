//! HUD example for `lava_ui_builder` — Challenge 6.
//!
//! Demonstrates UI anchored to screen corners overlaid on a 2D scene.
//!   Top-left:     minimap placeholder
//!   Bottom-left:  HP counter + animated health bar
//!   Bottom-center: game time + team scores
//!   Bottom-right: ammo counter
//!
//! On the scene (BSN) API: the HUD is one tree, corner-anchored with flexbox, and the
//! live numbers are ordinary marker-driven systems.
//!
//! Run with: `cargo run --example hud`

use bevy::prelude::*;
use lava_ui_builder::{scenes, LavaTheme, LavaUiPlugin, ProgressBar};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_plugins(LavaUiPlugin)
        .insert_resource(GameState::default())
        .add_systems(Startup, (setup_camera, setup_scene, setup_hud))
        .add_systems(
            Update,
            (
                tick_game,
                sync_hp_bar,
                sync_hp_text,
                sync_score_text,
                sync_ammo_text,
            ),
        )
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
        Self {
            hp: 80.0,
            hp_max: 100.0,
            elapsed: 0.0,
            score_alpha: 7,
            score_bravo: 5,
            ammo: 23,
            ammo_max: 30,
        }
    }
}

fn tick_game(time: Res<Time>, mut state: ResMut<GameState>) {
    state.elapsed += time.delta_secs();
    // HP slowly drains as a demo
    state.hp = f32::mul_add(time.delta_secs(), -2.0, state.hp).max(0.0);
}

// ── Scene ────────────────────────────────────────────────────────────────────

fn setup_scene(mut commands: Commands) {
    // Simple grid of colored quads as a fake game scene
    // `i8` so the grid coordinates convert to `f32` losslessly via `From`.
    for x in -4..=4_i8 {
        for y in -3..=3_i8 {
            let (fx, fy) = (f32::from(x), f32::from(y));
            let hue = (fx + fy) * 0.08;
            commands.spawn((
                Sprite {
                    color: Color::hsl(hue * 360.0, 0.4, 0.25),
                    custom_size: Some(Vec2::splat(80.0)),
                    ..default()
                },
                Transform::from_xyz(fx * 82.0, fy * 82.0, 0.0),
            ));
        }
    }
}

// ── UI marker components ──────────────────────────────────────────────────────

#[derive(Component, Default, Clone)]
struct HpBar;
#[derive(Component, Default, Clone)]
struct HpText;
#[derive(Component, Default, Clone)]
struct ScoreText;
#[derive(Component, Default, Clone)]
struct AmmoText;

// ── HUD ───────────────────────────────────────────────────────────────────────

const PANEL_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.7);
const PANEL_BORDER: Color = Color::srgb(0.3, 0.3, 0.4);

/// The HUD needs the initial `GameState`, so it is a scene-spawning system rather than a
/// `scene.spawn()` one.
fn setup_hud(mut commands: Commands, state: Res<GameState>) {
    commands.spawn_scene(hud(&state));
}

/// The shared smoked-glass panel the three corners use.
fn hud_panel() -> impl Scene {
    bsn! {
        scenes::column(4.0)
        Node {
            padding: {UiRect::all(px(12.0))},
            border: {UiRect::all(px(1.0))},
            border_radius: {BorderRadius::all(px(8.0))},
        }
        BackgroundColor(PANEL_BG)
        BorderColor::all(PANEL_BORDER)
    }
}

fn hud(state: &GameState) -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: {UiRect::all(px(16.0))},
        }
        Pickable::IGNORE
        Children [
            // Top row: minimap on the left.
            (
                scenes::row(0.0)
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Start,
                }
                Children [ minimap() ]
            ),
            // Bottom row: hp / score / ammo.
            (
                scenes::row(0.0)
                Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::End,
                }
                Children [
                    hp_panel(state),
                    score_panel(state),
                    ammo_panel(state),
                ]
            ),
        ]
    }
}

fn minimap() -> impl Scene {
    let blips: Vec<_> = [Color::srgb(0.3, 0.6, 1.0), Color::srgb(1.0, 0.5, 0.3)]
        .into_iter()
        .map(|color| {
            bsn! {
                Node { width: {px(8.0)}, height: {px(8.0)}, border_radius: {BorderRadius::MAX} }
                BackgroundColor(color)
            }
        })
        .collect();

    bsn! {
        scenes::column(0.0)
        Node {
            width: {px(160.0)},
            height: {px(160.0)},
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(6.0))},
        }
        BackgroundColor(Color::srgba(0.05, 0.15, 0.05, 0.92))
        BorderColor::all(Color::srgb(0.4, 0.6, 0.4))
        Children [
            scenes::text("[ MAP ]", 14.0, Color::srgb(0.5, 0.8, 0.5)),
            (
                scenes::row(20.0)
                Node { margin: {UiRect::top(px(10.0))} }
                Children [ {blips} ]
            ),
        ]
    }
}

fn hp_panel(state: &GameState) -> impl Scene {
    let hp_now = format!("{:.0}/{:.0}", state.hp, state.hp_max);
    let fraction = state.hp / state.hp_max;
    bsn! {
        hud_panel()
        Node { row_gap: {px(6.0)} }
        Children [
            (
                scenes::row(0.0)
                Node { width: {px(200.0)}, justify_content: JustifyContent::SpaceBetween }
                Children [
                    scenes::text("HP", 14.0, Color::srgb(0.8, 0.8, 0.8)),
                    (scenes::text(hp_now, 14.0, Color::srgb(0.5, 1.0, 0.5)) HpText),
                ]
            ),
            (
                scenes::progress_bar(
                    fraction, 200.0, 16.0,
                    Color::srgb(0.2, 0.85, 0.3),
                    Color::srgb(0.15, 0.15, 0.15),
                )
                HpBar
            ),
        ]
    }
}

fn score_panel(state: &GameState) -> impl Scene {
    let clock = game_time_str(state.elapsed);
    let score = format!("{} - {}", state.score_alpha, state.score_bravo);
    bsn! {
        hud_panel()
        Node { align_items: AlignItems::Center }
        Children [
            scenes::text(clock, 14.0, Color::srgb(0.9, 0.9, 0.9)),
            (
                scenes::row(16.0)
                Node { align_items: AlignItems::Center }
                Children [
                    scenes::text("ALPHA", 14.0, Color::srgb(0.4, 0.6, 1.0)),
                    (scenes::text(score, 22.0, Color::WHITE) ScoreText),
                    scenes::text("BRAVO", 14.0, Color::srgb(1.0, 0.5, 0.3)),
                ]
            ),
        ]
    }
}

fn ammo_panel(state: &GameState) -> impl Scene {
    let ammo = format!("{} / {}", state.ammo, state.ammo_max);
    let pips: Vec<_> = (0..state.ammo_max)
        .map(|i| {
            let color = if i < state.ammo {
                Color::srgb(1.0, 0.85, 0.2)
            } else {
                Color::srgb(0.2, 0.2, 0.2)
            };
            bsn! {
                Node { width: {px(6.0)}, height: {px(16.0)}, border_radius: {BorderRadius::all(px(2.0))} }
                BackgroundColor(color)
            }
        })
        .collect();

    bsn! {
        hud_panel()
        Node { align_items: AlignItems::End }
        Children [
            scenes::text("AMMO", 12.0, Color::srgb(0.7, 0.7, 0.7)),
            (scenes::text(ammo, 28.0, Color::WHITE) AmmoText),
            (
                scenes::row(3.0)
                Node { margin: {UiRect::top(px(4.0))} }
                Children [ {pips} ]
            ),
        ]
    }
}

// ── Runtime sync systems ──────────────────────────────────────────────────────

fn sync_hp_bar(state: Res<GameState>, mut bars: Query<&mut ProgressBar, With<HpBar>>) {
    if !state.is_changed() {
        return;
    }
    for mut bar in &mut bars {
        bar.value = state.hp / state.hp_max;
    }
}

fn sync_hp_text(state: Res<GameState>, mut texts: Query<&mut Text, With<HpText>>) {
    if !state.is_changed() {
        return;
    }
    for mut t in &mut texts {
        **t = format!("{:.0}/{:.0}", state.hp, state.hp_max);
    }
}

fn sync_score_text(state: Res<GameState>, mut texts: Query<&mut Text, With<ScoreText>>) {
    if !state.is_changed() {
        return;
    }
    for mut t in &mut texts {
        **t = format!("{} - {}", state.score_alpha, state.score_bravo);
    }
}

fn sync_ammo_text(state: Res<GameState>, mut texts: Query<&mut Text, With<AmmoText>>) {
    if !state.is_changed() {
        return;
    }
    for mut t in &mut texts {
        **t = format!("{} / {}", state.ammo, state.ammo_max);
    }
}

#[allow(clippy::cast_possible_truncation)]
fn game_time_str(elapsed: f32) -> String {
    let mins = (elapsed as i32) / 60;
    let secs = (elapsed as i32) % 60;
    format!("{mins:02}:{secs:02}")
}
