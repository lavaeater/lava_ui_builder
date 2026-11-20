//! Challenge 7: a bug report form.
//!
//! - A dropdown for the kind of bug (UI / cosmetic / gameplay).
//! - A one-line input for the title, a multi-line input for the description.
//! - Editing is Bevy's text input: a cursor moved with the arrow keys or a click,
//!   selection (shift+arrows, drag, double-click), and copy / cut / paste with the usual
//!   shortcuts.
//!
//! The fields write into a `Report` resource as you type; Submit checks it and answers
//! with a toast.
//!
//! Run with: `cargo run --example bug_report`

use bevy::prelude::*;
use lava_ui_builder::prelude::*;

const KINDS: [&str; 3] = ["UI", "Cosmetic", "Gameplay"];

#[derive(Resource, Default)]
struct Report {
    kind: usize,
    title: String,
    description: String,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::default())
        .init_resource::<Report>()
        .bind_resource::<Report>()
        .add_systems(Startup, form.spawn())
        .run();
}

fn form() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @scenes::screen()
        Children [
            @scenes::card()
            Node { width: px(520) }
            Children [
                @scenes::header("Report a bug")
                --
                @scenes::caption("What kind of bug?")
                --
                @scenes::select(&KINDS, 0)
                @scenes::bind_choice(|r: &Report| r.kind, |r, i| r.kind = i)
                --
                @scenes::caption("Title")
                --
                @scenes::text_input("One line that sums it up") AutoFocus
                @scenes::bind_input(|r: &mut Report, text| r.title = text.to_string())
                --
                @scenes::caption("What happened?")
                --
                @scenes::text_area("Steps to reproduce, what you expected, what you saw", 180.0)
                @scenes::bind_input(|r: &mut Report, text| r.description = text.to_string())
                --
                @scenes::row(10.0)
                Node { justify_content: JustifyContent::FlexEnd }
                Children [
                    @scenes::button("Submit") Variant::Success on(submit)
                ]
            ]
        ]
    }
}

fn submit(_: On<Activate>, report: Res<Report>, mut commands: Commands) {
    let toast = if report.title.trim().is_empty() {
        Toast::warning("Give the bug a title first.")
    } else {
        let kind = KINDS.get(report.kind).copied().unwrap_or("?");
        let lines = report.description.lines().count();
        Toast::success(format!("Sent [{kind}] \"{}\" ({lines} lines).", report.title.trim()))
    };
    commands.trigger(toast);
}
