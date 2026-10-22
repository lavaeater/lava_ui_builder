//! Text crawls (`lava_ui_builder::crawl`): a Star Wars scroll and a typewriter dialog.
//!
//! 1 = flat credits roll, 2 = tilted scroll, 3 = typewriter dialog, Space = advance / skip,
//! C = clear. Finished crawls are logged.
//!
//! Run with: `cargo run --example crawl`

use bevy::prelude::*;
use lava_ui_builder::crawl::{
    AdvanceCrawl, ClearCrawls, CrawlFinished, CrawlPlugin, CrawlSpec, CrawlStyle, ShowCrawl,
};
use lava_ui_builder::{LavaTheme, LavaUiPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin, CrawlPlugin))
        .insert_resource(LavaTheme::default())
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
            commands.spawn(Text::new(
                "1 flat roll   2 tilted scroll   3 typewriter   Space advance   C clear",
            ));
        })
        .add_systems(Update, (keys, log_finished))
        .run();
}

fn story() -> Vec<String> {
    [
        "It is a period of suburban unrest.",
        "Aliens, striking from a hidden cul-de-sac, have won their first victory against \
         the Homeowners' Association.",
        "",
        "During the battle, neighbourhood watch volunteers managed to steal secret plans to \
         the aliens' ultimate weapon, a lawn-flattening mothership with enough power to \
         destroy an entire street.",
    ]
    .map(String::from)
    .to_vec()
}

fn keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut show: MessageWriter<ShowCrawl>,
    mut advance: MessageWriter<AdvanceCrawl>,
    mut clear: MessageWriter<ClearCrawls>,
) {
    let scroll = |id: &str, tilt: f32| CrawlSpec {
        id: id.into(),
        title: "EPISODE IV: A NEW LAWN".into(),
        lines: story(),
        style: CrawlStyle::Scroll { tilt },
        speed: 60.0,
        ..CrawlSpec::default()
    };
    if keys.just_pressed(KeyCode::Digit1) {
        show.write(ShowCrawl(scroll("flat", 0.0)));
    }
    if keys.just_pressed(KeyCode::Digit2) {
        show.write(ShowCrawl(scroll("tilted", 1.5)));
    }
    if keys.just_pressed(KeyCode::Digit3) {
        show.write(ShowCrawl(CrawlSpec {
            id: "dialog".into(),
            title: "Mrs. Henderson".into(),
            lines: story(),
            style: CrawlStyle::Typewriter,
            speed: 40.0,
            backdrop: 0.2,
            hint: "Space".into(),
            ..CrawlSpec::default()
        }));
    }
    if keys.just_pressed(KeyCode::Space) {
        advance.write(AdvanceCrawl);
    }
    if keys.just_pressed(KeyCode::KeyC) {
        clear.write(ClearCrawls);
    }
}

fn log_finished(mut finished: MessageReader<CrawlFinished>) {
    for done in finished.read() {
        info!("crawl finished: {done:?}");
    }
}
