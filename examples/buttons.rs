//! Buttons showcase for `lava_ui_builder`.
//!
//! Demonstrates:
//! - `themed_button` bundle-function with observer actions
//! - `add_themed_button` imperative builder with `ButtonBuilder` customization
//! - Feathers-style buttons: default, primary, disabled, marked, custom corners
//! - `InteractionPalette` hover/press color system
//!
//! Run with: `cargo run --example buttons`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(Update, handle_interaction_palette)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Marker components for imperative buttons ─────────────────────────────

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct SettingsButton;

#[derive(Component)]
struct QuitButton;

// ── UI setup ─────────────────────────────────────────────────────────────

fn setup_ui(mut commands: Commands, theme: Res<LavaTheme>) {
    // ── Section 1: Bundle-function themed buttons ────────────────────
    // Spawn the root container, then add themed_button children individually
    // (themed_button uses SpawnWith internally, so we add them as children).
    let root = commands
        .spawn((
            ui_root_with(
                "Bundle Buttons",
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
            ),
        ))
        .id();

    let hdr = commands.spawn(header("Bundle Buttons", &theme.text)).id();
    let play = commands
        .spawn(themed_button(
            "Play",
            |_activate: On<Pointer<Click>>| {
                info!("Play clicked (bundle button)");
            },
            &theme.button,
        ))
        .id();
    let settings = commands
        .spawn(themed_button(
            "Settings",
            |_activate: On<Pointer<Click>>| {
                info!("Settings clicked (bundle button)");
            },
            &theme.button,
        ))
        .id();
    let close = commands
        .spawn(themed_button_small(
            "X",
            |_activate: On<Pointer<Click>>| {
                info!("Close clicked (small bundle button)");
            },
            &theme.button,
        ))
        .id();
    commands
        .entity(root)
        .add_children(&[hdr, play, settings, close]);

    // ── Section 2: Imperative UIBuilder buttons ──────────────────────
    let mut ui = UIBuilder::new(commands, Some(theme.clone()));

    ui.set_node(Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(50.0),
        height: Val::Percent(100.0),
        left: Val::Percent(50.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(16.0),
        ..default()
    });

    // Header
    ui.with_child(|h| {
        h.with_text("Imperative Buttons", Some(TextStyle::size(40.0)));
    });

    // Themed button via add_themed_button + ButtonBuilder
    ui.add_themed_button(PlayButton, |btn| {
        btn.text("Play")
            .size_px(200.0, 60.0)
            .border_radius_all_px(12.0);
    });

    ui.add_themed_button(SettingsButton, |btn| {
        btn.text("Settings")
            .size_px(200.0, 60.0)
            .bg_color(Color::srgb(0.4, 0.3, 0.6))
            .font_size(28.0);
    });

    ui.add_themed_button(QuitButton, |btn| {
        btn.text("Quit")
            .size_px(200.0, 60.0)
            .bg_color(Color::srgb(0.6, 0.15, 0.15));
    });

    // Feathers-style buttons
    ui.with_child(|spacer| {
        spacer.height_px(16.0);
    });
    ui.with_child(|h| {
        h.with_text("Feathers Buttons", Some(TextStyle::size(32.0)));
    });

    ui.feathers_button("Default", |_activate: On<Activate>| {
        info!("Feathers default button activated");
        external_function();
    });

    ui.feathers_button_primary("Primary", |_activate: On<Activate>| {
        info!("Feathers primary button activated");
    }, |_btn| {});

    ui.feathers_button_disabled("Disabled", |_activate: On<Activate>| {
        info!("This should not fire");
    });

    ui.build();
}

fn external_function() {
    info!("I was called");
}

// ── Runtime system: apply InteractionPalette colors on hover/press ───────

fn handle_interaction_palette(
    mut query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut bg) in &mut query {
        bg.0 = match interaction {
            Interaction::Pressed => palette.pressed,
            Interaction::Hovered => palette.hovered,
            Interaction::None => palette.none,
        };
    }
}