//! Health bar example for `lava_ui_builder` — Challenge 3, on the scene (BSN) API.
//!
//! A sphere you move with WASD / arrow keys carries a nameplate (name, health meter,
//! "7 / 10") anchored to it in world space. Health lives in the ECS on the sphere, starts
//! at 10 and drains by 1 per second; at 0 the sphere is despawned and the nameplate goes
//! with it.
//!
//! The library pieces:
//! - `scenes::meter` -- a progress bar whose colours are theme tokens (`HealthFill`,
//!   `MeterBg`), so the T key's theme swap repaints it live
//! - `WorldFollower` -- tracks the sphere's screen position, and despawns the nameplate
//!   when the sphere is gone; nothing here has to clean the UI up by hand
//!
//! WASD/arrows move · T toggles theme · Space respawns.
//!
//! Run with: `cargo run --example health_bar`

use bevy::prelude::*;
use lava_ui_builder::{
    ColorToken, LavaTheme, LavaUiPlugin, MeterTheme, ProgressBar, TextTheme, WorldFollower,
    scenes,
};

const MAX_HEALTH: u32 = 10;
const SPEED: f32 = 4.0;
/// The nameplate is a fixed width so `WorldFollower`'s offset can centre it.
const PLATE_WIDTH: f32 = 160.0;
/// Pixels above the sphere's centre where the nameplate's top edge sits.
const PLATE_LIFT: f32 = 120.0;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, (setup_world, setup_hint, spawn_character))
        .add_systems(
            Update,
            (
                move_character,
                drain_health,
                sync_nameplates,
                toggle_theme,
                respawn.run_if(no_character),
            ),
        )
        .run();
}

// ── Game state, all in the ECS ────────────────────────────────────────────────

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

impl Health {
    fn fraction(&self) -> f32 {
        self.current as f32 / self.max.max(1) as f32
    }

    fn label(&self) -> String {
        format!("{} / {}", self.current, self.max)
    }
}

/// Ticks health down once per second.
#[derive(Component)]
struct Drain(Timer);

/// On a nameplate: the character whose health it shows.
#[derive(Component)]
struct HealthOf(Entity);

/// The "7 / 10" line inside a nameplate.
#[derive(Component, Default, Clone)]
struct HealthText;

// ── Setup ─────────────────────────────────────────────────────────────────────

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 9.0, 11.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.30, 0.45, 0.30))),
    ));
}

fn setup_hint(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        Node { position_type: PositionType::Absolute, left: px(16), top: px(12) }
        Children [
            @scenes::label("WASD / arrows move  ·  T theme  ·  Space respawn")
        ]
    });
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let health = Health {
        current: MAX_HEALTH,
        max: MAX_HEALTH,
    };
    let plate = nameplate("Sphere", &health);

    let character = commands
        .spawn((
            Character,
            Name::new("Sphere"),
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.85, 0.35, 0.30))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            health,
            Drain(Timer::from_seconds(1.0, TimerMode::Repeating)),
        ))
        .id();

    commands.spawn_scene(plate).insert((
        WorldFollower {
            target: character,
            offset: Vec2::new(-PLATE_WIDTH / 2.0, -PLATE_LIFT),
        },
        HealthOf(character),
    ));
}

/// Name, health meter and "current / max", stacked and centred. Every colour is a token:
/// the text is `label`, the bar is `meter` on `HealthFill`.
fn nameplate(name: &str, health: &Health) -> impl Scene + use<> {
    let name = name.to_string();
    let value = health.fraction();
    let hp = health.label();
    bsn! {
        Node {
            width: {px(PLATE_WIDTH)},
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
        }
        Pickable::IGNORE
        Children [
            @scenes::label(name)
            --
            @scenes::meter(value, 110.0, 12.0, ColorToken::HealthFill)
            --
            @scenes::label(hp) HealthText
        ]
    }
}

// ── Game → UI ─────────────────────────────────────────────────────────────────

fn move_character(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut characters: Query<&mut Transform, With<Character>>,
) {
    let pressed = |a: KeyCode, b: KeyCode| keys.pressed(a) || keys.pressed(b);
    let mut dir = Vec3::ZERO;
    if pressed(KeyCode::KeyW, KeyCode::ArrowUp) {
        dir.z -= 1.0;
    }
    if pressed(KeyCode::KeyS, KeyCode::ArrowDown) {
        dir.z += 1.0;
    }
    if pressed(KeyCode::KeyA, KeyCode::ArrowLeft) {
        dir.x -= 1.0;
    }
    if pressed(KeyCode::KeyD, KeyCode::ArrowRight) {
        dir.x += 1.0;
    }
    let step = dir.normalize_or_zero() * SPEED * time.delta_secs();
    for mut transform in &mut characters {
        transform.translation += step;
    }
}

/// Lose 1 HP per second; at 0 the character is despawned. Its nameplate is not touched
/// here: `WorldFollower` despawns it once the target is gone.
fn drain_health(
    mut commands: Commands,
    time: Res<Time>,
    mut characters: Query<(Entity, &mut Health, &mut Drain)>,
) {
    for (entity, mut health, mut drain) in &mut characters {
        if !drain.0.tick(time.delta()).just_finished() {
            continue;
        }
        health.current = health.current.saturating_sub(1);
        if health.current == 0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Push each character's health into its nameplate's meter and text.
fn sync_nameplates(
    characters: Query<Ref<Health>>,
    plates: Query<(&HealthOf, &Children)>,
    mut bars: Query<&mut ProgressBar>,
    mut texts: Query<&mut Text, With<HealthText>>,
) {
    for (of, children) in &plates {
        let Ok(health) = characters.get(of.0) else {
            continue;
        };
        if !health.is_changed() {
            continue;
        }
        for child in children.iter() {
            if let Ok(mut bar) = bars.get_mut(child) {
                bar.value = health.fraction();
            } else if let Ok(mut text) = texts.get_mut(child) {
                **text = health.label();
            }
        }
    }
}

// ── Theme + respawn ───────────────────────────────────────────────────────────

/// One resource write; the meter, text and hint all repaint through their tokens.
fn toggle_theme(
    keys: Res<ButtonInput<KeyCode>>,
    mut theme: ResMut<LavaTheme>,
    mut light: Local<bool>,
) {
    if !keys.just_pressed(KeyCode::KeyT) {
        return;
    }
    *light = !*light;
    *theme = if *light {
        light_theme()
    } else {
        LavaTheme::default()
    };
}

fn light_theme() -> LavaTheme {
    LavaTheme {
        text: TextTheme {
            header_color: Color::srgb(0.10, 0.10, 0.18),
            label_color: Color::srgb(0.08, 0.08, 0.12),
            ..TextTheme::default()
        },
        meter: MeterTheme {
            bg: Color::srgba(1.0, 1.0, 1.0, 0.6),
            fill: Color::srgb(0.20, 0.35, 0.75),
            health: Color::srgb(0.90, 0.45, 0.10),
        },
        ..LavaTheme::default()
    }
}

fn no_character(characters: Query<(), With<Character>>) -> bool {
    characters.is_empty()
}

fn respawn(
    keys: Res<ButtonInput<KeyCode>>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
) {
    if keys.just_pressed(KeyCode::Space) {
        spawn_character(commands, meshes, materials);
    }
}
