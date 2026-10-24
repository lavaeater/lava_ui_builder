#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "a broken assumption in a test should panic loudly -- that is the failure signal"
)]

//! Headless checks for `lava_ui_builder::crawl`: a crawl asked for by message comes up,
//! queues the next one behind it, and reports `CrawlFinished` once it has faded out.
//! Time is stepped by hand so the real-time fades do not make the test wait.

use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lava_ui_builder::crawl::{
    AdvanceCrawl, ClearCrawls, CrawlFinished, CrawlPlugin, CrawlQueue, CrawlRoot, CrawlSpec,
    CrawlStyle, ShowCrawl,
};

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, CrawlPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
        )));
    app
}

fn dialog(id: &str, lines: &[&str]) -> CrawlSpec {
    CrawlSpec {
        id: id.into(),
        title: "Neighbour".into(),
        lines: lines.iter().map(ToString::to_string).collect(),
        style: CrawlStyle::Typewriter,
        speed: 100.0,
        ..CrawlSpec::default()
    }
}

fn finished(app: &mut App) -> Vec<CrawlFinished> {
    app.world_mut()
        .resource_mut::<Messages<CrawlFinished>>()
        .drain()
        .collect()
}

fn roots(app: &mut App) -> usize {
    app.world_mut()
        .query_filtered::<(), With<CrawlRoot>>()
        .iter(app.world())
        .count()
}

#[test]
fn crawls_show_one_at_a_time_and_report_when_done() {
    let mut app = test_app();
    app.world_mut()
        .write_message(ShowCrawl(dialog("first", &["hi"])));
    app.world_mut()
        .write_message(ShowCrawl(dialog("second", &["bye"])));
    app.update();

    assert_eq!(roots(&mut app), 1, "one crawl on screen");
    let queue = app.world().resource::<CrawlQueue>();
    assert_eq!(queue.active().map(|s| s.id.as_str()), Some("first"));
    assert!(queue.is_modal());

    // Past the fade-in and typing, then turn the only page.
    for _ in 0..10 {
        app.update();
    }
    app.world_mut().write_message(AdvanceCrawl);
    // Messages only live two frames: collect them as the fade-out runs.
    let mut done = Vec::new();
    for _ in 0..10 {
        app.update();
        done.extend(finished(&mut app));
    }
    assert_eq!(
        done.first(),
        Some(&CrawlFinished {
            id: "first".into(),
            skipped: false
        })
    );
    app.update();
    let queue = app.world().resource::<CrawlQueue>();
    assert_eq!(queue.active().map(|s| s.id.as_str()), Some("second"));
    assert_eq!(roots(&mut app), 1, "the first root is gone");
}

#[test]
fn clearing_drops_everything_without_a_finish() {
    let mut app = test_app();
    app.world_mut()
        .write_message(ShowCrawl(dialog("a", &["x"])));
    app.world_mut()
        .write_message(ShowCrawl(dialog("b", &["y"])));
    app.update();
    app.world_mut().write_message(ClearCrawls);
    app.update();
    app.update();

    assert!(!app.world().resource::<CrawlQueue>().is_busy());
    assert_eq!(roots(&mut app), 0);
    assert!(finished(&mut app).is_empty());
}
