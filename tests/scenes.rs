#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a broken assumption in a test should panic loudly -- that is the failure signal"
)]

//! Headless checks for the BSN scene functions in `lava_ui_builder::scenes`.
//!
//! These spawn each scene into a minimal `App` and assert on the entities that come out,
//! which is the part `cargo run --example` cannot tell you at a glance: that the tree has
//! the shape the widget promises, and that the `#Name` entity references inside
//! `collapsible` resolve to the real section entity.

use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::scene::ScenePlugin;
use bevy::ui_widgets::Button as WidgetsButton;
use lava_ui_builder::{
    scenes, tokens, CollapseToggleButton, Collapsible, CollapsibleContent, ColorToken,
    InteractionPalette, LavaTheme, ProgressBar, ProgressBarFill, ThemedTextColor,
};
use bevy::ui::ScrollPosition;

/// Bevy's scene resolution needs an asset server and the scene plugin; the token system
/// plus a `LavaTheme` are what turn the widgets' tokens into concrete colors. Nothing
/// here touches rendering or windowing.
fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), ScenePlugin))
        .insert_resource(LavaTheme::default())
        .add_systems(Update, tokens::apply_theme_tokens);
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

#[test]
fn a_label_resolves_its_token_to_the_theme_color() {
    let mut app = test_app();
    let entity = spawn(&mut app, scenes::label("hello"));

    let expected = LavaTheme::default().text.label_color;
    let world = app.world();
    assert_eq!(
        world.get::<Text>(entity).map(|t| t.0.as_str()),
        Some("hello")
    );
    assert_eq!(world.get::<TextColor>(entity).map(|c| c.0), Some(expected));
}

/// The whole point of tokens: a live entity repaints when the theme resource changes,
/// with no despawn/respawn.
#[test]
fn changing_the_theme_repaints_existing_entities() {
    let mut app = test_app();
    let entity = spawn(&mut app, scenes::label("hello"));
    assert_eq!(
        app.world().get::<TextColor>(entity).map(|c| c.0),
        Some(LavaTheme::default().text.label_color)
    );

    let recolored = Color::srgb(0.01, 0.02, 0.03);
    let mut theme = LavaTheme::default();
    theme.text.label_color = recolored;
    app.world_mut().insert_resource(theme);
    app.update();

    assert_eq!(
        app.world().get::<TextColor>(entity).map(|c| c.0),
        Some(recolored),
        "the same entity must follow the theme without being rebuilt"
    );
}

#[test]
fn a_button_is_a_headless_button_with_a_palette_and_a_text_child() {
    let mut app = test_app();
    let entity = spawn(&mut app, scenes::button("Play"));

    let world = app.world();
    assert!(
        world.get::<WidgetsButton>(entity).is_some(),
        "must be a ui_widgets button, so Activate fires and Pressed/Hovered are tracked"
    );
    // Without this pair the palette system has nothing to read -- the bug this migration
    // started from.
    let palette = world
        .get::<InteractionPalette>(entity)
        .expect("ThemedPalette requires and fills an InteractionPalette");
    let theme = LavaTheme::default();
    assert_eq!(palette.none, theme.button.bg);
    assert_eq!(palette.hovered, theme.button.bg_hovered);
    assert!(world.get::<bevy::picking::hover::Hovered>(entity).is_some());

    let children = world
        .get::<Children>(entity)
        .expect("button has a caption child");
    assert_eq!(children.len(), 1);
    assert_eq!(
        world.get::<Text>(children[0]).map(|t| t.0.as_str()),
        Some("Play")
    );
}

/// Overriding a themed widget means patching its *token*, not the color the token
/// produced -- the token system would repaint over a bare `TextColor`.
#[test]
fn a_caller_patch_overrides_the_token_and_leaves_the_rest() {
    let mut app = test_app();
    let entity = spawn(
        &mut app,
        bsn! {
            scenes::label("hello")
            ThemedTextColor(ColorToken::HeaderText)
        },
    );

    let world = app.world();
    assert_eq!(
        world.get::<TextColor>(entity).map(|c| c.0),
        Some(LavaTheme::default().text.header_color)
    );
    assert_eq!(
        world.get::<Text>(entity).map(|t| t.0.as_str()),
        Some("hello"),
        "patching the token must not disturb the text set by the scene"
    );
}

#[test]
fn a_progress_bar_sizes_its_fill_child_from_the_value() {
    let mut app = test_app();
    let entity = spawn(
        &mut app,
        scenes::progress_bar(0.25, 200.0, 10.0, Color::WHITE, Color::BLACK),
    );

    let world = app.world();
    assert_eq!(world.get::<ProgressBar>(entity).map(|b| b.value), Some(0.25));
    let fill = world.get::<Children>(entity).expect("fill child")[0];
    assert!(world.get::<ProgressBarFill>(fill).is_some());
    assert_eq!(
        world.get::<Node>(fill).map(|n| n.width),
        Some(Val::Percent(25.0))
    );
}

#[test]
fn a_collapsible_resolves_its_name_references_to_the_section_entity() {
    let mut app = test_app();
    let section = spawn(
        &mut app,
        scenes::collapsible("Details", false, bsn_list![scenes::label("inner")]),
    );

    let world = app.world();
    assert!(world.get::<Collapsible>(section).is_some());
    let children = world.get::<Children>(section).expect("toggle + content");
    assert_eq!(children.len(), 2);

    // This is the assertion that matters: `#Section` inside the widget's `bsn!` has to
    // come back out as the real entity id on both children.
    let toggle = world
        .get::<CollapseToggleButton>(children[0])
        .expect("first child is the toggle");
    assert_eq!(toggle.target, section);
    let content = world
        .get::<CollapsibleContent>(children[1])
        .expect("second child is the content");
    assert_eq!(content.parent, section);
}

#[test]
fn a_collapsed_section_hides_its_content_and_shows_the_collapsed_arrow() {
    let mut app = test_app();
    let section = spawn(&mut app, scenes::collapsible("Details", true, bsn_list![]));

    let world = app.world();
    let children = world.get::<Children>(section).unwrap();
    assert_eq!(
        world.get::<Node>(children[1]).map(|n| n.display),
        Some(Display::None)
    );
    let toggle_text = world.get::<Children>(children[0]).unwrap()[0];
    assert!(world
        .get::<Text>(toggle_text)
        .is_some_and(|t| t.0.starts_with('\u{25b6}')));
}

#[test]
fn a_selected_list_item_uses_the_selected_palette() {
    let mut app = test_app();
    let unselected = spawn(&mut app, scenes::list_item("one", false));
    let selected = spawn(&mut app, scenes::list_item("two", true));

    let world = app.world();
    let a = world.get::<InteractionPalette>(unselected).unwrap();
    let b = world.get::<InteractionPalette>(selected).unwrap();
    assert_ne!(
        a.none, b.none,
        "selection has to be visible without hovering"
    );
    // Not theme-driven: the token system must leave these alone.
    assert!(world.get::<lava_ui_builder::ThemedPalette>(unselected).is_none());
}

#[test]
fn a_scrollable_list_can_actually_scroll() {
    let mut app = test_app();
    let entity = spawn(&mut app, scenes::scrollable_list(2.0));

    let world = app.world();
    // `Overflow::scroll_y` alone does nothing without this component, and
    // `handle_scroll_input` only sees entities that have it.
    assert!(world.get::<ScrollPosition>(entity).is_some());
    assert_eq!(
        world.get::<Node>(entity).map(|n| n.overflow),
        Some(Overflow::scroll_y())
    );
}

#[test]
fn a_bounded_scrollable_list_patches_the_cap_over_the_plain_one() {
    let mut app = test_app();
    let entity = spawn(&mut app, scenes::scrollable_list_bounded(2.0, 120.0));

    let node = app.world().get::<Node>(entity).cloned().unwrap();
    assert_eq!(node.max_height, Val::Px(120.0));
    assert_eq!(node.flex_grow, 0.0, "the cap replaces the grow");
    assert_eq!(
        node.overflow,
        Overflow::scroll_y(),
        "and keeps what the composed scene set"
    );
}

#[test]
fn replace_children_swaps_the_contents_and_keeps_the_parent() {
    let mut app = test_app();
    let parent = spawn(
        &mut app,
        bsn! {
            scenes::column(2.0)
            Children [ scenes::label("old a"), scenes::label("old b") ]
        },
    );
    assert_eq!(app.world().get::<Children>(parent).unwrap().len(), 2);

    let mut commands = app.world_mut().commands();
    scenes::replace_children(
        &mut commands,
        parent,
        bsn_list![
            scenes::label("new a"),
            scenes::label("new b"),
            scenes::label("new c"),
        ],
    );
    app.update();

    let world = app.world();
    assert!(
        world.get_entity(parent).is_ok(),
        "the parent entity must survive a rebuild"
    );
    let children = world.get::<Children>(parent).unwrap();
    assert_eq!(children.len(), 3);
    assert_eq!(
        world.get::<Text>(children[0]).map(|t| t.0.as_str()),
        Some("new a")
    );
}
