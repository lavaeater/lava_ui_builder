//! Challenge 4: a responsive menu with nine-slice buttons.
//!
//! - The buttons are drawn from one nine-slice frame image (`assets/ui/frame.png`), tinted
//!   by the theme, so the skin follows variants, hover, press and focus like flat buttons.
//! - An image sits to the right of the buttons.
//! - Normal widths: the menu is centred. At 400 px and below: the buttons fill the screen
//!   width and the image moves above them.
//!
//! Resize the window narrow to see it switch. All of it is `Responsive` overrides on two
//! nodes; there is no layout code.
//!
//! Run with: `cargo run --example responsive_menu`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const NARROW: f32 = 400.0;
const SKIN: &str = "ui/frame.png";

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme {
            // Scale UI against a smaller reference width than the 1920 default, so the
            // menu stays legible as the window narrows.
            ui_width: 900.0,
            ..LavaTheme::default()
        })
        .add_systems(Startup, scene.spawn())
        .run();
}

fn scene() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @scenes::screen()
        Children [
            // Row on wide screens; on narrow ones a column, reversed so the image (the
            // second child) ends up on top.
            @scenes::row(40.0)
            Node { align_items: AlignItems::Center }
            Responsive {
                breakpoint: {NARROW},
                narrow: NarrowLayout {
                    flex_direction: {Some(FlexDirection::ColumnReverse)},
                    width: {Some(percent(100))},
                    padding: {Some(UiRect::all(px(16.0)))},
                    row_gap: {Some(px(20.0))},
                    align_items: {Some(AlignItems::Stretch)},
                },
            }
            Children [
                @scenes::column(12.0)
                Node { width: px(280), align_items: AlignItems::Stretch }
                @scenes::full_width_below(NARROW)
                Children [
                    @scenes::skinned_button("Play", SKIN, 12.0) AutoFocus
                    --
                    @scenes::skinned_button("Options", SKIN, 12.0) Variant::Secondary
                    --
                    @scenes::skinned_button("Credits", SKIN, 12.0) Variant::Secondary
                    --
                    @scenes::skinned_button("Quit", SKIN, 12.0) Variant::Danger
                ]
                --
                @scenes::image("hero.png")
                Node { width: px(300), align_self: AlignSelf::Center }
            ]
        ]
    }
}
