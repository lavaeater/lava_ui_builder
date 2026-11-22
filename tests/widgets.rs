#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::float_cmp,
    reason = "a broken assumption in a test should panic loudly -- that is the failure signal"
)]

//! Headless checks for the layer added on top of the basic scenes: variants and states,
//! theme presets, bindings, the form controls, screens, dialogs and toasts.
//!
//! Like `tests/scenes.rs` these run without a window or renderer, so they cover state and
//! entity structure; layout, focus movement and looks are checked by running the
//! examples.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy::state::app::StatesPlugin;
use bevy::ui::{Checked, InteractionDisabled, Selected};
use bevy::ui_widgets::{Activate, SliderValue, ValueChange};
use lava_ui_builder::controls::{ControlsPlugin, Cycle, OptionIndex, Select, StepCycle};
use lava_ui_builder::overlay::{Dialog, OverlayPlugin, ToastLife};
use lava_ui_builder::prelude::*;
use lava_ui_builder::tokens::ThemeTokensPlugin;
use lava_ui_builder::{InteractionPalette, VariantText};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ScenePlugin,
        ThemeTokensPlugin,
        ControlsPlugin,
    ))
    .insert_resource(LavaTheme::default());
    app
}

fn spawn(app: &mut App, scene: impl Scene) -> Entity {
    let entity = app
        .world_mut()
        .spawn_scene(scene)
        .expect("scene should resolve without pending asset dependencies")
        .id();
    app.update();
    entity
}

fn palette(app: &App, entity: Entity) -> InteractionPalette {
    app.world()
        .get::<InteractionPalette>(entity)
        .cloned()
        .expect("has a palette")
}

fn caption_color(app: &App, button: Entity) -> Color {
    let children = app.world().get::<Children>(button).expect("has a caption");
    let caption = children
        .iter()
        .find(|c| app.world().get::<VariantText>(*c).is_some())
        .expect("caption carries VariantText");
    app.world().get::<TextColor>(caption).unwrap().0
}

// ── variants and states ─────────────────────────────────────────────────────────────

#[test]
fn a_patched_variant_changes_the_palette_and_the_caption() {
    let mut app = test_app();
    let button = spawn(
        &mut app,
        bsn! { @scenes::button("Quit") Variant::Danger },
    );
    let danger = LavaTheme::default().button.danger;
    assert_eq!(palette(&app, button).none, danger.bg);
    assert_eq!(palette(&app, button).hovered, danger.hovered);
    assert_eq!(caption_color(&app, button), danger.text);
}

/// Selection and disabling are Bevy's own markers: inserting one repaints, removing it
/// restores the variant -- no palette juggling in game code.
#[test]
fn state_markers_override_the_variant_and_removing_them_restores_it() {
    let mut app = test_app();
    let button = spawn(&mut app, scenes::button("Medium"));
    let theme = LavaTheme::default();

    app.world_mut().entity_mut(button).insert(Selected);
    app.update();
    assert_eq!(palette(&app, button).none, theme.button.selected.bg);
    assert_eq!(caption_color(&app, button), theme.button.selected.text);

    app.world_mut().entity_mut(button).insert(InteractionDisabled);
    app.update();
    assert_eq!(
        palette(&app, button).none,
        theme.button.disabled.bg,
        "disabled wins over selected"
    );
    assert_eq!(
        palette(&app, button).hovered,
        theme.button.disabled.bg,
        "and does not react to hover"
    );

    app.world_mut()
        .entity_mut(button)
        .remove::<(Selected, InteractionDisabled)>();
    app.update();
    assert_eq!(palette(&app, button).none, theme.button.bg);
}

#[test]
fn a_variant_follows_a_theme_swap() {
    let mut app = test_app();
    let button = spawn(&mut app, bsn! { @scenes::button("Ok") Variant::Success });
    let accent = Color::srgb(0.9, 0.2, 0.6);
    let mut theme = LavaTheme::default();
    theme.button.success = ButtonColors::from_base(accent, Color::BLACK);
    app.world_mut().insert_resource(theme);
    app.update();
    assert_eq!(palette(&app, button).none, accent);
    assert_eq!(caption_color(&app, button), Color::BLACK);
}

#[test]
fn a_selected_list_item_starts_with_the_selected_marker() {
    let mut app = test_app();
    let row = spawn(&mut app, scenes::list_item("two", true));
    assert!(app.world().get::<Selected>(row).is_some());
    assert_eq!(
        palette(&app, row).none,
        LavaTheme::default().button.selected.bg
    );
}

// ── theme presets ───────────────────────────────────────────────────────────────────

#[test]
fn the_light_preset_inverts_text_against_its_surfaces() {
    let light = LavaTheme::light();
    let dark = LavaTheme::dark();
    assert_ne!(light.text.label_color, dark.text.label_color);
    assert_ne!(light.surface.raised, dark.surface.raised);
}

#[test]
fn from_accent_drives_the_primary_button_and_the_fills() {
    let accent = Color::srgb(0.85, 0.25, 0.45);
    let theme = LavaTheme::from_accent(accent);
    assert_eq!(theme.button.bg, accent);
    assert_eq!(theme.controls.fill, accent);
    assert_eq!(theme.meter.fill, accent);
    assert_ne!(theme.button.bg_hovered, accent, "hover is derived, not equal");
}

#[test]
fn from_accent_picks_a_readable_caption() {
    let pale = LavaTheme::from_accent(Color::srgb(0.95, 0.95, 0.80));
    let deep = LavaTheme::from_accent(Color::srgb(0.10, 0.10, 0.45));
    assert_ne!(pale.button.text_color, deep.button.text_color);
}

#[test]
fn new_font_tokens_resolve() {
    let mut app = test_app();
    let title = spawn(&mut app, scenes::title("Lava"));
    let size = app.world().get::<TextFont>(title).unwrap().font_size;
    assert_eq!(size, FontSize::from(LavaTheme::default().text.title_size));
}

// ── bindings ────────────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct Hud {
    ammo: u32,
    shield: f32,
    paused: bool,
}

#[test]
fn bound_text_is_written_on_spawn_and_when_the_resource_changes() {
    let mut app = test_app();
    app.insert_resource(Hud {
        ammo: 30,
        ..default()
    })
    .bind_resource::<Hud>();
    let label = spawn(
        &mut app,
        bsn! { @scenes::label("") @scenes::bind_text(|h: &Hud| format!("{} rounds", h.ammo)) },
    );
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "30 rounds");

    app.world_mut().resource_mut::<Hud>().ammo = 29;
    app.update();
    assert_eq!(app.world().get::<Text>(label).unwrap().0, "29 rounds");
}

#[test]
fn a_bound_meter_and_visibility_follow_the_resource() {
    let mut app = test_app();
    app.insert_resource(Hud {
        shield: 0.25,
        ..default()
    })
    .bind_resource::<Hud>();
    let meter = spawn(
        &mut app,
        bsn! {
            @scenes::meter(1.0, 100.0, 8.0, ColorToken::MeterFill)
            @scenes::bind_meter(|h: &Hud| h.shield)
        },
    );
    let banner = spawn(
        &mut app,
        bsn! { @scenes::label("PAUSED") @scenes::bind_visible(|h: &Hud| h.paused) },
    );
    app.update();
    assert_eq!(app.world().get::<ProgressBar>(meter).unwrap().value, 0.25);
    assert_eq!(
        app.world().get::<Node>(banner).unwrap().display,
        Display::None
    );

    app.world_mut().resource_mut::<Hud>().paused = true;
    app.update();
    assert_eq!(
        app.world().get::<Node>(banner).unwrap().display,
        Display::Flex
    );
}

#[derive(Component)]
struct Health(f32);

#[test]
fn a_component_binding_reads_another_entity() {
    let mut app = test_app();
    app.bind_component::<Health>();
    let hero = app.world_mut().spawn(Health(7.0)).id();
    let plate = spawn(
        &mut app,
        bsn! {
            @scenes::label("")
            @scenes::bind_text_to(hero, |h: &Health| format!("{:.0} HP", h.0))
        },
    );
    app.update();
    assert_eq!(app.world().get::<Text>(plate).unwrap().0, "7 HP");

    app.world_mut().get_mut::<Health>(hero).unwrap().0 = 6.0;
    app.update();
    assert_eq!(app.world().get::<Text>(plate).unwrap().0, "6 HP");
}

#[derive(Resource, Default)]
struct Settings {
    volume: f32,
    subtitles: bool,
    quality: usize,
}

#[test]
fn a_bound_slider_reads_the_resource_and_writes_changes_back() {
    let mut app = test_app();
    app.insert_resource(Settings {
        volume: 0.3,
        ..default()
    })
    .bind_resource::<Settings>();
    let slider = spawn(
        &mut app,
        bsn! {
            @scenes::slider(0.0, 1.0, 0.0)
            @scenes::bind_slider(|s: &Settings| s.volume, |s, v| s.volume = v)
        },
    );
    app.update();
    assert_eq!(app.world().get::<SliderValue>(slider).unwrap().0, 0.3);

    app.world_mut().trigger(ValueChange {
        source: slider,
        value: 0.8_f32,
        is_final: true,
    });
    app.update();
    assert_eq!(app.world().resource::<Settings>().volume, 0.8);
}

#[test]
fn a_bound_toggle_mirrors_the_flag_both_ways() {
    let mut app = test_app();
    app.init_resource::<Settings>().bind_resource::<Settings>();
    let toggle = spawn(
        &mut app,
        bsn! {
            @scenes::toggle("Subtitles", true)
            @scenes::bind_checked(|s: &Settings| s.subtitles, |s, on| s.subtitles = on)
        },
    );
    app.update();
    assert!(
        app.world().get::<Checked>(toggle).is_none(),
        "the resource wins over the scene's initial state"
    );

    app.world_mut().trigger(ValueChange {
        source: toggle,
        value: true,
        is_final: true,
    });
    app.update();
    assert!(app.world().resource::<Settings>().subtitles);
}

// ── controls ────────────────────────────────────────────────────────────────────────

#[test]
fn a_choice_checks_one_option_and_reports_its_index() {
    let mut app = test_app();
    app.init_resource::<Settings>().bind_resource::<Settings>();
    let choice = spawn(
        &mut app,
        bsn! {
            @scenes::choice(&["Low", "Medium", "High"], 1)
            @scenes::bind_choice(|s: &Settings| s.quality, |s, i| s.quality = i)
        },
    );
    app.update();
    let options: Vec<Entity> = app.world().get::<Children>(choice).unwrap().to_vec();
    let checked: Vec<bool> = options
        .iter()
        .map(|o| app.world().get::<Checked>(*o).is_some())
        .collect();
    assert_eq!(checked, [true, false, false], "bound to quality 0");

    // What the radio group emits when "High" is clicked.
    app.world_mut().trigger(ValueChange {
        source: choice,
        value: options[2],
        is_final: true,
    });
    app.update();
    assert_eq!(app.world().resource::<Settings>().quality, 2);
    assert!(app.world().get::<Checked>(options[2]).is_some());
    assert!(app.world().get::<Checked>(options[0]).is_none());
}

#[test]
fn a_cycle_wraps_and_updates_its_caption() {
    let mut app = test_app();
    let cycle = spawn(&mut app, scenes::cycle(&["Easy", "Normal", "Hard"], 2));

    app.world_mut().trigger(StepCycle {
        entity: cycle,
        delta: 1,
    });
    app.update();
    assert_eq!(app.world().get::<Cycle>(cycle).unwrap().index, 0, "wraps");

    let caption = app
        .world()
        .get::<Children>(cycle)
        .unwrap()
        .iter()
        .find_map(|c| app.world().get::<Text>(c).map(|t| t.0.clone()))
        .unwrap();
    assert_eq!(caption, "Easy");
}

#[test]
fn a_select_opens_picks_and_closes() {
    let mut app = test_app();
    let select = spawn(&mut app, scenes::select(&["UI", "Cosmetic", "Gameplay"], 0));
    let children: Vec<Entity> = app.world().get::<Children>(select).unwrap().to_vec();
    let (button, popup) = (children[0], children[1]);

    app.world_mut().trigger(Activate { entity: button });
    app.update();
    assert!(app.world().get::<Select>(select).unwrap().open);
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::Flex
    );

    let third = app
        .world()
        .get::<Children>(popup)
        .unwrap()
        .iter()
        .find(|r| app.world().get::<OptionIndex>(*r) == Some(&OptionIndex(2)))
        .unwrap();
    app.world_mut().trigger(Activate { entity: third });
    app.update();
    app.update();
    let state = app.world().get::<Select>(select).unwrap();
    assert_eq!((state.index, state.open), (2, false));
    assert!(app.world().get::<Selected>(third).is_some());
    assert_eq!(
        app.world().get::<Node>(popup).unwrap().display,
        Display::None
    );
}

#[test]
fn a_checkbox_starts_checked_when_asked() {
    let mut app = test_app();
    let on = spawn(&mut app, scenes::checkbox("Vsync", true));
    let off = spawn(&mut app, scenes::checkbox("Vsync", false));
    assert!(app.world().get::<Checked>(on).is_some());
    assert!(app.world().get::<Checked>(off).is_none());
}

// ── screens ─────────────────────────────────────────────────────────────────────────

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Menu {
    #[default]
    Main,
    Audio,
}

#[derive(Component, Default, Clone)]
struct MainMarker;
#[derive(Component, Default, Clone)]
struct AudioMarker;

fn main_screen() -> impl Scene {
    bsn! { MainMarker Children [ @scenes::button("Audio") @scenes::go_to(Menu::Audio) ] }
}
fn audio_screen() -> impl Scene {
    bsn! { AudioMarker }
}

#[test]
fn screens_spawn_on_enter_despawn_on_exit_and_go_back() {
    let mut app = test_app();
    app.add_plugins(StatesPlugin)
        .init_state::<Menu>()
        .add_screen(Menu::Main, main_screen)
        .add_screen(Menu::Audio, audio_screen)
        .screen_back(Menu::Audio, Menu::Main);
    app.update();
    fn count<T: Component>(app: &mut App) -> usize {
        app.world_mut().query::<&T>().iter(app.world()).count()
    }
    assert_eq!(count::<MainMarker>(&mut app), 1);

    let button = app
        .world_mut()
        .query_filtered::<Entity, With<GoTo<Menu>>>()
        .single(app.world())
        .unwrap();
    app.world_mut().trigger(Activate { entity: button });
    app.update();
    app.update();
    assert_eq!(count::<MainMarker>(&mut app), 0, "the old screen despawns");
    assert_eq!(count::<AudioMarker>(&mut app), 1);

    app.world_mut().trigger(NavigateBack);
    app.update();
    app.update();
    assert_eq!(*app.world().resource::<State<Menu>>().get(), Menu::Main);
    assert_eq!(count::<AudioMarker>(&mut app), 0);
}

// ── overlays ────────────────────────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct Answer(Option<bool>);

#[test]
fn a_confirm_dialog_reports_its_answer_and_despawns() {
    let mut app = test_app();
    app.add_plugins(OverlayPlugin).init_resource::<Answer>();
    let dialog = app
        .world_mut()
        .spawn_scene(scenes::confirm("Quit?", "Progress is lost.", "Quit"))
        .unwrap()
        .observe(|r: On<DialogResult>, mut answer: ResMut<Answer>| {
            answer.0 = Some(r.confirmed);
        })
        .id();
    app.update();
    assert!(app.world().get::<Dialog>(dialog).is_some());

    let confirm_button = app
        .world_mut()
        .query::<(Entity, &lava_ui_builder::overlay::DialogAnswer)>()
        .iter(app.world())
        .find(|(_, a)| a.0)
        .map(|(e, _)| e)
        .unwrap();
    app.world_mut().trigger(Activate {
        entity: confirm_button,
    });
    app.update();
    assert_eq!(app.world().resource::<Answer>().0, Some(true));
    assert!(app.world().get_entity(dialog).is_err(), "despawned");
}

#[test]
fn a_toast_appears_and_expires() {
    let mut app = test_app();
    app.add_plugins(OverlayPlugin);
    app.world_mut().trigger(Toast {
        seconds: 0.05,
        ..Toast::success("Saved")
    });
    app.update();
    let toasts = app
        .world_mut()
        .query::<&ToastLife>()
        .iter(app.world())
        .count();
    assert_eq!(toasts, 1);
    // Toasts run on real time, so wait it out for real.
    std::thread::sleep(std::time::Duration::from_millis(80));
    app.update();
    app.update();
    let toasts = app
        .world_mut()
        .query::<&ToastLife>()
        .iter(app.world())
        .count();
    assert_eq!(toasts, 0);
}

/// Press Tab "at" whatever has focus, the way Bevy's input dispatch would.
fn press_tab(app: &mut App) {
    use bevy::input::ButtonState;
    use bevy::input::keyboard::{Key, KeyboardInput};
    use bevy::input_focus::{FocusedInput, InputFocus};
    // A real `Window`: focused input bubbles up to it and stops there.
    let window = app.world_mut().spawn(Window::default()).id();
    let target = app.world().resource::<InputFocus>().get().unwrap_or(window);
    let input = KeyboardInput {
        key_code: KeyCode::Tab,
        logical_key: Key::Tab,
        state: ButtonState::Pressed,
        text: None,
        repeat: false,
        window,
    };
    app.world_mut()
        .trigger(FocusedInput::new(target, input, window));
    app.update();
}

fn tab_app() -> App {
    let mut app = test_app();
    app.add_plugins((
        bevy::input::InputPlugin,
        bevy::input_focus::InputFocusPlugin,
        lava_ui_builder::focus::FocusPlugin,
    ));
    app.finish();
    app
}

fn two_buttons() -> impl Scene {
    bsn! {
        @scenes::screen()
        Children [
            @scenes::button("One") #One
            --
            @scenes::button("Two") #Two
        ]
    }
}

#[test]
fn tab_moves_focus_between_widgets_on_a_screen() {
    use bevy::input_focus::{FocusCause, InputFocus};
    let mut app = tab_app();
    let root = spawn(&mut app, two_buttons());
    let buttons: Vec<Entity> = app.world().get::<Children>(root).unwrap().iter().collect();
    // No renderer, so nothing computes visibility, and Tab skips hidden widgets.
    for entity in [root, buttons[0], buttons[1]] {
        app.world_mut()
            .entity_mut(entity)
            .insert(InheritedVisibility::VISIBLE);
    }
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(buttons[0], FocusCause::Navigated);
    press_tab(&mut app);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(buttons[1]));
}

#[test]
fn tab_with_nothing_focused_leaves_the_ui_alone() {
    use bevy::input_focus::InputFocus;
    let mut app = tab_app();
    spawn(&mut app, two_buttons());
    press_tab(&mut app);
    assert_eq!(
        app.world().resource::<InputFocus>().get(),
        None,
        "Tab is the game's until a widget has focus"
    );
}
