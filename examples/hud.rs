//! Challenge 6: a HUD over a 3D scene.
//!
//! - Top left: a minimap image.  Bottom left: HP counter and bar.
//! - Bottom centre: game time and both teams' scores.  Bottom right: ammo and a reload
//!   warning.
//!
//! Each corner is `scenes::anchored(..)`; every number is bound to the one `Match`
//! resource, so the only game-side system is the one that plays the match. Space fires,
//! R reloads.
//!
//! Run with: `cargo run --example hud`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

#[derive(Resource)]
struct Match {
    hp: f32,
    hp_max: f32,
    seconds: f32,
    score: (u32, u32),
    ammo: u32,
    ammo_max: u32,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .insert_resource(Match {
            hp: 100.0,
            hp_max: 100.0,
            seconds: 0.0,
            score: (7, 5),
            ammo: 30,
            ammo_max: 30,
        })
        .bind_resource::<Match>()
        .add_systems(Startup, (world, hud.spawn()))
        .add_systems(Update, play)
        .run();
}

fn hud() -> impl SceneList {
    bsn_list! {
        @scenes::fullscreen()
        Children [
            @scenes::anchored(ScreenAnchor::TopLeft, 16.0)
            Children [ @scenes::icon("minimap.png", 160.0) ]
            --
            @scenes::anchored(ScreenAnchor::BottomLeft, 16.0)
            Children [
                @scenes::header("") @scenes::bind_text(|m: &Match| format!("{:.0} HP", m.hp))
                --
                @scenes::meter(1.0, 220.0, 14.0, ColorToken::HealthFill)
                @scenes::bind_meter(|m: &Match| m.hp / m.hp_max)
            ]
            --
            @scenes::anchored(ScreenAnchor::Bottom, 16.0)
            Children [
                @scenes::label("") @scenes::bind_text(|m: &Match| {
                    let s = m.seconds as u32;
                    format!("{:02}:{:02}", s / 60, s % 60)
                })
                --
                @scenes::header("") @scenes::bind_text(|m: &Match| format!("{}  -  {}", m.score.0, m.score.1))
            ]
            --
            @scenes::anchored(ScreenAnchor::BottomRight, 16.0)
            Children [
                @scenes::caption("RELOAD [R]") ThemedTextColor(ColorToken::Danger)
                @scenes::bind_visible(|m: &Match| m.ammo < 8)
                --
                @scenes::header("") @scenes::bind_text(|m: &Match| format!("{} / {}", m.ammo, m.ammo_max))
                --
                @scenes::meter(1.0, 160.0, 8.0, ColorToken::MeterFill)
                @scenes::bind_meter(|m: &Match| m.ammo as f32 / m.ammo_max as f32)
            ]
        ]
    }
}

fn play(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Match>) {
    game.seconds += time.delta_secs();
    game.hp = (game.hp - time.delta_secs() * 2.0).max(0.0);
    if keys.just_pressed(KeyCode::Space) && game.ammo > 0 {
        game.ammo -= 1;
        if game.ammo % 5 == 0 {
            game.score.0 += 1;
        }
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.ammo = game.ammo_max;
    }
}

/// A plain 3D scene for the HUD to sit on.
fn world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-6.0, 5.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(30.0, 30.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.25, 0.32, 0.25))),
    ));
    for i in 0..6_u8 {
        let x = f32::from(i) * 2.5 - 6.0;
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(1.2, 1.0 + f32::from(i % 3), 1.2))),
            MeshMaterial3d(materials.add(Color::srgb(0.55, 0.50, 0.45))),
            Transform::from_xyz(x, 0.5, -2.0 - f32::from(i % 2) * 3.0),
        ));
    }
}
