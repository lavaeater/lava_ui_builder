//! Challenge 3: a health bar anchored to a character in world space.
//!
//! - A sphere you move with WASD / arrow keys.
//! - Its name, a health meter and "7 / 10" float above it (`WorldFollower`).
//! - Health lives in the ECS on the sphere: it starts at 10 and drops by 1 per second.
//! - At 0 the sphere is despawned, and the nameplate goes with it (`WorldFollower`
//!   despawns a follower whose target is gone).
//!
//! The nameplate reads the sphere's `Health` through component bindings, so nothing
//! copies health into the UI. T swaps to the light theme; Space respawns.
//!
//! Run with: `cargo run --example health_bar`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const SPEED: f32 = 4.0;
const PLATE_WIDTH: f32 = 160.0;

#[derive(Component)]
struct Character;

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

#[derive(Component)]
struct Drain(Timer);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .bind_component::<Health>()
        .add_systems(Startup, (world, spawn_character))
        .add_systems(
            Update,
            (
                move_character,
                drain_health,
                toggle_theme,
                spawn_character.run_if(no_character_and_space),
            ),
        )
        .run();
}

/// Name, meter and "current / max", centred over the character.
fn nameplate(character: Entity) -> impl Scene {
    bsn! {
        Node {
            width: {px(PLATE_WIDTH)},
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(4),
        }
        Pickable::IGNORE
        Children [
            @scenes::label("Sphere")
            --
            @scenes::meter(1.0, 110.0, 12.0, ColorToken::HealthFill)
            @scenes::bind_meter_to(character, |h: &Health| h.current as f32 / h.max as f32)
            --
            @scenes::caption("")
            @scenes::bind_text_to(character, |h: &Health| format!("{} / {}", h.current, h.max))
        ]
    }
}

fn spawn_character(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let character = commands
        .spawn((
            Character,
            Health { current: 10, max: 10 },
            Drain(Timer::from_seconds(1.0, TimerMode::Repeating)),
            Mesh3d(meshes.add(Sphere::new(0.5))),
            MeshMaterial3d(materials.add(Color::srgb(0.85, 0.35, 0.30))),
            Transform::from_xyz(0.0, 0.5, 0.0),
        ))
        .id();
    commands.spawn_scene(nameplate(character)).insert(WorldFollower {
        target: character,
        offset: Vec2::new(-PLATE_WIDTH / 2.0, -120.0),
    });
}

fn drain_health(
    mut commands: Commands,
    time: Res<Time>,
    mut characters: Query<(Entity, &mut Health, &mut Drain)>,
) {
    for (entity, mut health, mut drain) in &mut characters {
        if drain.0.tick(time.delta()).just_finished() {
            health.current = health.current.saturating_sub(1);
            if health.current == 0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn move_character(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut characters: Query<&mut Transform, With<Character>>,
) {
    let axis = |neg: [KeyCode; 2], pos: [KeyCode; 2]| {
        f32::from(u8::from(keys.any_pressed(pos))) - f32::from(u8::from(keys.any_pressed(neg)))
    };
    let x = axis([KeyCode::KeyA, KeyCode::ArrowLeft], [KeyCode::KeyD, KeyCode::ArrowRight]);
    let z = axis([KeyCode::KeyW, KeyCode::ArrowUp], [KeyCode::KeyS, KeyCode::ArrowDown]);
    let step = Vec3::new(x, 0.0, z).normalize_or_zero() * SPEED * time.delta_secs();
    for mut transform in &mut characters {
        transform.translation += step;
    }
}

fn toggle_theme(keys: Res<ButtonInput<KeyCode>>, mut theme: ResMut<LavaTheme>, mut light: Local<bool>) {
    if keys.just_pressed(KeyCode::KeyT) {
        *light = !*light;
        *theme = if *light { LavaTheme::light() } else { LavaTheme::dark() };
    }
}

fn no_character_and_space(keys: Res<ButtonInput<KeyCode>>, characters: Query<(), With<Character>>) -> bool {
    keys.just_pressed(KeyCode::Space) && characters.is_empty()
}

fn world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 9.0, 11.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight { shadow_maps_enabled: true, ..default() },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.30, 0.45, 0.30))),
    ));
    commands.spawn_scene(bsn! {
        @scenes::anchored(ScreenAnchor::TopLeft, 16.0)
        Children [ @scenes::caption("WASD / arrows move  -  T theme  -  Space respawns") ]
    });
}
