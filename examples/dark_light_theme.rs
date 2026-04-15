//! Dark/Light Theme example for `lava_ui_builder` — Challenge 9.
//!
//! Demonstrates the `LavaTheme` system: a single toggle button swaps between
//! a dark and a light preset, rebuilding the UI via `start_from_entity`.
//!
//! Run with: `cargo run --example dark_light_theme`

use bevy::prelude::*;
use lava_ui_builder::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(dark_theme())
        .add_plugins(LavaUiPlugin)
        .init_resource::<ThemeMode>()
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(Update, on_theme_changed.run_if(resource_changed::<ThemeMode>))
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

// ── Theme presets ─────────────────────────────────────────────────────────────

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy)]
enum ThemeMode { #[default] Dark, Light }

fn dark_theme() -> LavaTheme { LavaTheme::default() }

fn light_theme() -> LavaTheme {
    LavaTheme {
        button: ButtonTheme {
            bg:                  Color::srgb(0.72, 0.72, 0.88),
            bg_hovered:          Color::srgb(0.60, 0.60, 0.76),
            bg_pressed:          Color::srgb(0.50, 0.50, 0.66),
            text_color:          Color::srgb(0.10, 0.10, 0.15),
            border_color:        Color::srgb(0.50, 0.50, 0.65),
            collapsible_bg:      Color::srgb(0.75, 0.75, 0.82),
            collapsible_bg_hovered: Color::srgb(0.65, 0.65, 0.72),
            collapsible_bg_pressed: Color::srgb(0.55, 0.70, 0.55),
            ..ButtonTheme::default()
        },
        text: TextTheme {
            header_color: Color::srgb(0.10, 0.10, 0.18),
            label_color:  Color::srgb(0.22, 0.22, 0.30),
            ..TextTheme::default()
        },
        bg_color:     Color::srgb(0.88, 0.88, 0.94),
        border_color: Color::srgb(0.55, 0.55, 0.68),
        ..LavaTheme::default()
    }
}

// ── Marker component ──────────────────────────────────────────────────────────

#[derive(Component)] struct UiRoot;

// ── Initial UI build ──────────────────────────────────────────────────────────

fn setup_ui(mut commands: Commands, theme: Res<LavaTheme>) {
    let root = commands.spawn((Name::new("UI Root"), UiRoot, Node::default())).id();
    let mut ui = UIBuilder::start_from_entity(commands, root, false, Some(theme.clone()));
    populate_ui(&mut ui, &theme);
    ui.build();
}

fn populate_ui(ui: &mut UIBuilder, theme: &LavaTheme) {
    let is_light = theme.bg_color.to_srgba().red > 0.5;
    let toggle_label = if is_light { "Switch to Dark" } else { "Switch to Light" };

    ui.set_node(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(24.0),
        ..default()
    });
    ui.insert_bundle(BackgroundColor(theme.bg_color));

    // Title
    ui.with_child(|h| {
        h.with_text("Theme Showcase", Some(TextStyle {
            font_size: Some(theme.text.header_size),
            color: Some(theme.text.header_color),
            ..default()
        }));
    });

    // Sample card
    ui.with_child(|card| {
        card.display_flex().flex_column().gap_px(12.0)
            .padding_all_px(24.0)
            .bg_color(theme.bg_color.mix(&Color::BLACK, 0.08))
            .border_all_px(2.0, theme.border_color)
            .border_radius_all_px(12.0)
            .width_px(400.0);

        card.add_text_child("Sample Panel", Some(TextStyle::size_color(22.0, theme.text.header_color)));
        card.add_text_child(
            "All colors adapt when the theme changes.\nNo individual component updates needed.",
            Some(TextStyle { font_size: Some(16.0), color: Some(theme.text.label_color), ..default() }),
        );

        for label in ["Option A", "Option B", "Option C"] {
            card.add_button_observe(label,
                |btn| { btn.size(Val::Percent(100.0), Val::Px(44.0)); },
                move |_: On<Activate>| info!("Selected: {}", label));
        }
    });

    // Theme toggle button
    ui.add_button_observe(toggle_label,
        |btn| { btn.size_px(220.0, 52.0); },
        |_: On<Activate>, mut mode: ResMut<ThemeMode>| {
            *mode = if *mode == ThemeMode::Dark { ThemeMode::Light } else { ThemeMode::Dark };
        });
}

// ── Rebuild on theme change ───────────────────────────────────────────────────

fn on_theme_changed(
    commands: Commands,
    mode: Res<ThemeMode>,
    mut lava_theme: ResMut<LavaTheme>,
    ui_root: Query<Entity, With<UiRoot>>,
) {
    let new_theme = if *mode == ThemeMode::Light { light_theme() } else { dark_theme() };
    *lava_theme = new_theme.clone();

    if let Ok(root) = ui_root.single() {
        let mut ui = UIBuilder::start_from_entity(commands, root, true, Some(new_theme.clone()));
        populate_ui(&mut ui, &new_theme);
        ui.build();
    }
}
