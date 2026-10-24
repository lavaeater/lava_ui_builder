//! Text crawls: a block of story text shown over the game, either rolling up the screen
//! (the Star Wars opening, with an optional perspective tilt) or typed into a dialog box
//! one page at a time.
//!
//! The module knows nothing about the host game. A crawl is plain data ([`CrawlSpec`]);
//! the host asks for one with [`ShowCrawl`] (or [`CrawlQueue::push`]), forwards its own
//! "skip / next" input as [`AdvanceCrawl`], and listens for [`CrawlFinished`]. Crawls
//! queue: one shows at a time, in the order they were asked for.
//!
//! ```ignore
//! app.add_plugins(CrawlPlugin);
//! // ...
//! show.write(ShowCrawl(CrawlSpec {
//!     id: "intro".into(),
//!     title: "Episode IV".into(),
//!     lines: vec!["It is a period of civil war.".into()],
//!     ..CrawlSpec::default()
//! }));
//! ```
//!
//! Crawls run on `Time<Real>`, so a host that pauses `Time<Virtual>` while a crawl is up
//! (see [`CrawlSpec::modal`]) still sees it play. Colours and the font come from
//! [`LavaTheme`]'s text theme (title = header colour, body = label colour), read every
//! frame, so a theme swap repaints a crawl in flight. They are written as plain
//! `TextColor`s rather than tokens because the crawl fades them.

use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::text::{Justify, TextLayout};

use crate::LavaTheme;

/// Seconds the crawl takes to fade in, and before which [`AdvanceCrawl`] is ignored (so
/// the press that triggered a crawl does not also skip it).
pub const FADE_IN_SECS: f32 = 0.5;
/// Seconds the crawl takes to fade out once it is done or skipped.
pub const FADE_OUT_SECS: f32 = 0.6;
/// The largest [`CrawlStyle::Scroll`] tilt. Past this the far text is a few pixels tall.
pub const MAX_TILT: f32 = 4.0;
/// Fraction of the screen height, from the top, over which scrolling text fades out.
const TOP_FADE: f32 = 0.2;
/// Gap between paragraphs, in logical pixels at scale 1.
const PARAGRAPH_GAP: f32 = 24.0;
/// Draws over everything else the host has on screen.
const CRAWL_Z: i32 = 900;
const BOX_BG: Color = Color::srgba(0.03, 0.04, 0.07, 0.92);

/// How the text is presented.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CrawlStyle {
    /// The paragraphs roll up the screen at `speed` logical pixels per second, centred.
    /// `tilt` lays them on a plane leaning away from the viewer: 0 is a flat credits
    /// roll, 1 a gentle Star Wars, larger values shrink the far text faster (capped at
    /// [`MAX_TILT`]). The title, if any, rolls first.
    Scroll { tilt: f32 },
    /// A box at the bottom of the screen; each paragraph is one page, typed in at `speed`
    /// characters per second. The title sits above the text like a speaker's name.
    Typewriter,
}

impl Default for CrawlStyle {
    fn default() -> Self {
        Self::Scroll { tilt: 1.0 }
    }
}

/// One crawl. Every field has a default, so a data file only lists what it needs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CrawlSpec {
    /// Echoed back in [`CrawlFinished`]. Hosts loading crawls from files usually fill it
    /// from the file name.
    pub id: String,
    /// Shown first (scroll) or above the text (typewriter). Empty = none.
    pub title: String,
    /// The paragraphs. An empty string is a blank line in a scroll.
    pub lines: Vec<String>,
    pub style: CrawlStyle,
    /// Scroll: logical pixels per second. Typewriter: characters per second.
    pub speed: f32,
    /// Body text size in logical pixels; the title is scaled from it.
    pub font_size: f32,
    /// Opacity of the black backdrop behind the crawl: 0 = see-through, 1 = black.
    pub backdrop: f32,
    /// Whether [`AdvanceCrawl`] can cut a scroll short. A typewriter always advances, a
    /// page at a time.
    pub skippable: bool,
    /// Typewriter: seconds a finished page stays up before turning by itself. 0 = wait
    /// for [`AdvanceCrawl`].
    pub auto_advance: f32,
    /// A small prompt drawn in the corner of the typewriter box once a page is complete,
    /// e.g. "Fire to continue". Empty = none.
    pub hint: String,
    /// A hint to the host that the crawl should hold up the game (pause, close input).
    /// The crawl itself does nothing with it; read it through [`CrawlQueue::active`].
    pub modal: bool,
}

impl Default for CrawlSpec {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: String::new(),
            lines: Vec::new(),
            style: CrawlStyle::default(),
            speed: 40.0,
            font_size: 30.0,
            backdrop: 0.6,
            skippable: true,
            auto_advance: 0.0,
            hint: String::new(),
            modal: true,
        }
    }
}

impl CrawlSpec {
    /// Characters on typewriter page `page` (0 past the end).
    fn page_len(&self, page: usize) -> usize {
        self.lines.get(page).map_or(0, |line| line.chars().count())
    }

    fn tilt(&self) -> f32 {
        match self.style {
            CrawlStyle::Scroll { tilt } => tilt.clamp(0.0, MAX_TILT),
            CrawlStyle::Typewriter => 0.0,
        }
    }
}

/// Ask for a crawl. It queues behind any crawl already showing.
#[derive(Message, Clone, Debug)]
pub struct ShowCrawl(pub CrawlSpec);

/// The host's "next" input: completes the typewriter page being typed, turns a finished
/// page, or cuts a skippable scroll short. Ignored during the fade-in.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct AdvanceCrawl;

/// Drop the crawl on screen and everything queued, without [`CrawlFinished`]. For a host
/// leaving the screen the crawls belong to.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct ClearCrawls;

/// A crawl ran to its end (or was skipped) and has faded out.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct CrawlFinished {
    pub id: String,
    pub skipped: bool,
}

/// The crawl on screen and the ones waiting behind it.
#[derive(Resource, Debug, Default)]
pub struct CrawlQueue {
    active: Option<ActiveCrawl>,
    pending: VecDeque<CrawlSpec>,
}

#[derive(Debug)]
struct ActiveCrawl {
    spec: CrawlSpec,
    progress: CrawlProgress,
    root: Entity,
}

impl CrawlQueue {
    /// Queue a crawl. The same as writing [`ShowCrawl`], but visible to
    /// [`is_busy`](Self::is_busy) straight away.
    pub fn push(&mut self, spec: CrawlSpec) {
        self.pending.push_back(spec);
    }

    /// The crawl on screen, if any.
    pub fn active(&self) -> Option<&CrawlSpec> {
        self.active.as_ref().map(|a| &a.spec)
    }

    /// A crawl is showing or waiting to.
    pub fn is_busy(&self) -> bool {
        self.active.is_some() || !self.pending.is_empty()
    }

    /// The crawl on screen -- or, between crawls, the next in line -- holds up the game
    /// ([`CrawlSpec::modal`]). Looking ahead keeps a host from letting the game run for
    /// the frame between two modal crawls, and lets it stop before the first one appears.
    pub fn is_modal(&self) -> bool {
        self.active()
            .or_else(|| self.pending.front())
            .is_some_and(|spec| spec.modal)
    }
}

/// Where a crawl is in its life. Pure: [`tick`](Self::tick) and
/// [`advance`](Self::advance) never touch the world, so the timing is testable.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CrawlProgress {
    /// Real seconds since the crawl appeared.
    pub elapsed: f32,
    /// Scroll: logical pixels the text has travelled.
    pub scroll: f32,
    /// Scroll: the `scroll` at which the last line has left the top of the screen. Known
    /// once the text has been laid out; the scroll cannot end before then.
    pub scroll_end: Option<f32>,
    /// Typewriter: the page showing.
    pub page: usize,
    /// Typewriter: characters typed on it (fractional between frames).
    pub typed: f32,
    /// Typewriter: seconds the current page has been complete.
    pub page_done_for: f32,
    /// `Some(seconds)` once fading out.
    pub closing: Option<f32>,
    pub skipped: bool,
}

impl CrawlProgress {
    /// Overall opacity: the fade-in times the fade-out.
    pub fn fade(&self) -> f32 {
        let fade_in = (self.elapsed / FADE_IN_SECS).clamp(0.0, 1.0);
        let fade_out = self
            .closing
            .map_or(1.0, |t| 1.0 - (t / FADE_OUT_SECS).clamp(0.0, 1.0));
        fade_in * fade_out
    }

    /// Typewriter: every character of the current page is showing.
    pub fn page_complete(&self, spec: &CrawlSpec) -> bool {
        self.typed as usize >= spec.page_len(self.page)
    }

    /// Characters to show on the current page.
    pub fn visible_chars(&self, spec: &CrawlSpec) -> usize {
        (self.typed as usize).min(spec.page_len(self.page))
    }

    fn close(&mut self, skipped: bool) {
        if self.closing.is_none() {
            self.closing = Some(0.0);
            self.skipped = skipped;
        }
    }

    fn turn_page(&mut self, spec: &CrawlSpec) {
        if self.page + 1 < spec.lines.len() {
            self.page += 1;
            self.typed = 0.0;
            self.page_done_for = 0.0;
        } else {
            self.close(false);
        }
    }

    /// The host's "next" input. See [`AdvanceCrawl`].
    pub fn advance(&mut self, spec: &CrawlSpec) {
        if self.elapsed < FADE_IN_SECS || self.closing.is_some() {
            return;
        }
        match spec.style {
            CrawlStyle::Scroll { .. } => {
                if spec.skippable {
                    self.close(true);
                }
            }
            CrawlStyle::Typewriter => {
                if self.page_complete(spec) {
                    self.turn_page(spec);
                } else {
                    self.typed = spec.page_len(self.page) as f32;
                }
            }
        }
    }

    /// Move time on by `dt` real seconds. Returns true once the crawl has faded out.
    pub fn tick(&mut self, dt: f32, spec: &CrawlSpec) -> bool {
        self.elapsed += dt;
        if let Some(t) = &mut self.closing {
            *t += dt;
            return *t >= FADE_OUT_SECS;
        }
        match spec.style {
            CrawlStyle::Scroll { .. } => {
                self.scroll += spec.speed * dt;
                if self.scroll_end.is_some_and(|end| self.scroll >= end) {
                    self.close(false);
                }
            }
            CrawlStyle::Typewriter => {
                if spec.lines.is_empty() {
                    self.close(false);
                } else if self.page_complete(spec) {
                    self.page_done_for += dt;
                    if spec.auto_advance > 0.0 && self.page_done_for >= spec.auto_advance {
                        self.turn_page(spec);
                    }
                } else {
                    self.typed += spec.speed * dt;
                }
            }
        }
        false
    }
}

/// The scroll's perspective. `depth` is how far a point has travelled up the plane from
/// the bottom of a screen `height` tall (both in logical pixels); returns where it is
/// drawn (y down from the top of the screen) and how much it is scaled.
///
/// The scale falls off as `D / (depth + D)`, with `D = height / tilt`, and the drawn
/// position is the integral of that scale, so a line's drawn height always matches its
/// scale and paragraphs never gap or overlap as they shrink. `tilt` 0 is a flat roll.
pub fn project(depth: f32, height: f32, tilt: f32) -> (f32, f32) {
    if tilt <= 0.0 || depth <= 0.0 {
        return (height - depth, 1.0);
    }
    let d = height / tilt;
    (height - d * (depth / d).ln_1p(), d / (depth + d))
}

/// The `depth` (see [`project`]) at which a point leaves the top of the screen.
pub fn exit_depth(height: f32, tilt: f32) -> f32 {
    if tilt <= 0.0 {
        return height;
    }
    let d = height / tilt;
    d * (height / d).exp_m1()
}

/// Splits `text` after its first `chars` characters, on a character boundary.
pub fn split_at_chars(text: &str, chars: usize) -> (&str, &str) {
    let at = text
        .char_indices()
        .nth(chars)
        .map_or(text.len(), |(i, _)| i);
    text.split_at_checked(at).unwrap_or((text, ""))
}

/// Registers the crawl messages, the [`CrawlQueue`] and the systems that show crawls.
pub struct CrawlPlugin;

impl Plugin for CrawlPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CrawlQueue>()
            .add_message::<ShowCrawl>()
            .add_message::<AdvanceCrawl>()
            .add_message::<ClearCrawls>()
            .add_message::<CrawlFinished>()
            .add_systems(
                Update,
                (
                    clear_crawls,
                    enqueue_crawls,
                    start_next_crawl,
                    tick_crawl,
                    draw_scroll,
                    draw_typewriter,
                )
                    .chain(),
            );
    }
}

/// The root of a crawl on screen.
#[derive(Component, Debug)]
pub struct CrawlRoot;

/// One paragraph of a scroll; `index` 0 is the title when there is one.
#[derive(Component, Debug)]
struct ScrollLine {
    index: usize,
    title: bool,
}

#[derive(Component, Debug)]
struct TypewriterBox;
#[derive(Component, Debug)]
struct TypewriterTitle;
#[derive(Component, Debug)]
struct TypewriterShown;
#[derive(Component, Debug)]
struct TypewriterHidden;
#[derive(Component, Debug)]
struct TypewriterHint;

fn clear_crawls(
    mut reader: MessageReader<ClearCrawls>,
    mut queue: ResMut<CrawlQueue>,
    mut commands: Commands,
) {
    if reader.read().count() == 0 {
        return;
    }
    queue.pending.clear();
    if let Some(active) = queue.active.take() {
        commands.entity(active.root).despawn();
    }
}

fn enqueue_crawls(mut reader: MessageReader<ShowCrawl>, mut queue: ResMut<CrawlQueue>) {
    for ShowCrawl(spec) in reader.read() {
        queue.push(spec.clone());
    }
}

fn start_next_crawl(
    mut queue: ResMut<CrawlQueue>,
    theme: Option<Res<LavaTheme>>,
    mut commands: Commands,
) {
    if queue.active.is_some() {
        return;
    }
    let Some(spec) = queue.pending.pop_front() else {
        return;
    };
    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let root = match spec.style {
        CrawlStyle::Scroll { .. } => spawn_scroll(&mut commands, &spec, &theme),
        CrawlStyle::Typewriter => spawn_typewriter(&mut commands, &spec, &theme),
    };
    queue.active = Some(ActiveCrawl {
        spec,
        progress: CrawlProgress::default(),
        root,
    });
}

/// A full-screen, click-through layer with the backdrop, clipped so scrolling text does
/// not spill.
fn crawl_root(name: &str) -> impl Bundle {
    (
        Name::new(format!("Crawl {name}")),
        CrawlRoot,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            overflow: Overflow::clip(),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(CRAWL_Z),
        Pickable::IGNORE,
    )
}

/// Text starts transparent: the draw systems fade it in once the layout is known.
fn crawl_text(text: &str, size: f32, theme: &LavaTheme) -> impl Bundle {
    (
        Text::new(text),
        TextFont::default()
            .with_font(theme.text.font.clone())
            .with_font_size(size),
        TextColor(Color::NONE),
    )
}

fn spawn_scroll(commands: &mut Commands, spec: &CrawlSpec, theme: &LavaTheme) -> Entity {
    let has_title = !spec.title.is_empty();
    let paragraphs = has_title
        .then_some((spec.title.as_str(), true))
        .into_iter()
        .chain(spec.lines.iter().map(|line| (line.as_str(), false)));
    let lines: Vec<(String, bool)> = paragraphs
        .map(|(text, title)| {
            // An empty Text has no height; a space keeps the blank line.
            let text = if text.is_empty() { " " } else { text };
            (text.to_string(), title)
        })
        .collect();
    let size = spec.font_size;
    let theme = theme.clone();
    commands
        .spawn(crawl_root(&spec.id))
        .with_children(|root| {
            root.spawn((
                Name::new("Crawl column"),
                Node {
                    position_type: PositionType::Absolute,
                    top: px(0.0),
                    width: percent(60.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(PARAGRAPH_GAP),
                    ..default()
                },
            ))
            .with_children(|column| {
                for (index, (text, title)) in lines.into_iter().enumerate() {
                    let font = if title { size * 1.6 } else { size };
                    column.spawn((
                        ScrollLine { index, title },
                        crawl_text(&text, font, &theme),
                        TextLayout::justify(Justify::Center),
                        UiTransform::IDENTITY,
                    ));
                }
            });
        })
        .id()
}

fn spawn_typewriter(commands: &mut Commands, spec: &CrawlSpec, theme: &LavaTheme) -> Entity {
    let size = spec.font_size;
    let title = spec.title.clone();
    let hint = spec.hint.clone();
    let theme = theme.clone();
    commands
        .spawn(crawl_root(&spec.id))
        .insert(Node {
            position_type: PositionType::Absolute,
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::bottom(percent(6.0)),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Name::new("Crawl box"),
                TypewriterBox,
                Node {
                    width: percent(70.0),
                    max_width: px(1100.0),
                    min_height: px(size * 5.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10.0),
                    padding: UiRect::all(px(24.0)),
                    border: UiRect::all(px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                BorderColor::all(Color::NONE),
            ))
            .with_children(|panel| {
                if !title.is_empty() {
                    panel.spawn((TypewriterTitle, crawl_text(&title, size * 0.8, &theme)));
                }
                panel
                    .spawn(crawl_text("", size, &theme))
                    .with_children(|text| {
                        text.spawn((
                            TypewriterShown,
                            TextSpan::new(""),
                            TextFont::default()
                                .with_font(theme.text.font.clone())
                                .with_font_size(size),
                            TextColor(Color::NONE),
                        ));
                        // The untyped rest of the page, invisible, so the typed part is
                        // laid out where it will finally sit and words do not jump lines.
                        text.spawn((
                            TypewriterHidden,
                            TextSpan::new(""),
                            TextFont::default()
                                .with_font(theme.text.font.clone())
                                .with_font_size(size),
                            TextColor(Color::NONE),
                        ));
                    });
                if !hint.is_empty() {
                    panel.spawn((
                        TypewriterHint,
                        crawl_text(&hint, size * 0.55, &theme),
                        Node {
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                    ));
                }
            });
        })
        .id()
}

fn tick_crawl(
    time: Res<Time<Real>>,
    mut advance: MessageReader<AdvanceCrawl>,
    mut queue: ResMut<CrawlQueue>,
    mut finished: MessageWriter<CrawlFinished>,
    mut commands: Commands,
) {
    let advances = advance.read().count();
    let Some(active) = queue.active.as_mut() else {
        return;
    };
    for _ in 0..advances {
        active.progress.advance(&active.spec);
    }
    if !active.progress.tick(time.delta_secs(), &active.spec) {
        return;
    }
    commands.entity(active.root).despawn();
    finished.write(CrawlFinished {
        id: active.spec.id.clone(),
        skipped: active.progress.skipped,
    });
    queue.active = None;
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let base = color.alpha();
    color.with_alpha(base * alpha)
}

/// Lays the scroll's paragraphs on the tilted plane: each line's flat position comes from
/// the summed heights above it, then [`project`] moves and scales it there.
fn draw_scroll(
    mut queue: ResMut<CrawlQueue>,
    theme: Option<Res<LavaTheme>>,
    mut roots: Query<(&ComputedNode, &mut BackgroundColor), With<CrawlRoot>>,
    mut lines: Query<(&ScrollLine, &ComputedNode, &mut UiTransform, &mut TextColor)>,
    mut heights: Local<Vec<(usize, f32)>>,
) {
    let Some(active) = queue.active.as_mut() else {
        return;
    };
    if !matches!(active.spec.style, CrawlStyle::Scroll { .. }) {
        return;
    }
    let Ok((root_node, mut backdrop)) = roots.get_mut(active.root) else {
        return;
    };
    let fade = active.progress.fade();
    backdrop.0 = Color::BLACK.with_alpha(active.spec.backdrop * fade);

    let scale = root_node.inverse_scale_factor();
    let height = root_node.size().y * scale;
    heights.clear();
    heights.extend(
        lines
            .iter()
            .map(|(line, node, _, _)| (line.index, node.size().y * scale)),
    );
    heights.sort_by_key(|(index, _)| *index);
    // Not laid out yet: keep everything hidden rather than flash it at the top.
    if height <= 0.0 || heights.iter().any(|(_, h)| *h <= 0.0) {
        return;
    }

    let tilt = active.spec.tilt();
    let mut tops = Vec::with_capacity(heights.len());
    let mut top = 0.0;
    for (_, h) in &*heights {
        tops.push(top);
        top += h + PARAGRAPH_GAP;
    }
    let content = (top - PARAGRAPH_GAP).max(0.0);
    active.progress.scroll_end = Some(content + exit_depth(height, tilt));

    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let offset = active.progress.scroll;
    for (line, node, mut transform, mut color) in &mut lines {
        let line_top = tops.get(line.index).copied().unwrap_or(0.0);
        let center = line_top + node.size().y * scale / 2.0;
        // Depth up the plane from the screen's bottom edge.
        let depth = offset - center;
        let (y, s) = project(depth, height, tilt);
        transform.translation = Val2::px(0.0, y - center);
        transform.scale = Vec2::splat(s);
        let top_fade = (y / (height * TOP_FADE)).clamp(0.0, 1.0);
        let base = if line.title {
            theme.text.header_color
        } else {
            theme.text.label_color
        };
        color.0 = with_alpha(base, fade * top_fade);
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn draw_typewriter(
    queue: Res<CrawlQueue>,
    theme: Option<Res<LavaTheme>>,
    time: Res<Time<Real>>,
    mut roots: Query<&mut BackgroundColor, (With<CrawlRoot>, Without<TypewriterBox>)>,
    mut boxes: Query<(&mut BackgroundColor, &mut BorderColor), With<TypewriterBox>>,
    mut titles: Query<&mut TextColor, (With<TypewriterTitle>, Without<TypewriterShown>)>,
    mut shown: Query<
        (&mut TextSpan, &mut TextColor),
        (With<TypewriterShown>, Without<TypewriterHidden>),
    >,
    mut hidden: Query<&mut TextSpan, (With<TypewriterHidden>, Without<TypewriterShown>)>,
    mut hints: Query<
        &mut TextColor,
        (
            With<TypewriterHint>,
            Without<TypewriterTitle>,
            Without<TypewriterShown>,
        ),
    >,
) {
    let Some(active) = queue.active.as_ref() else {
        return;
    };
    if active.spec.style != CrawlStyle::Typewriter {
        return;
    }
    let theme = theme.map(|t| t.clone()).unwrap_or_default();
    let (spec, progress) = (&active.spec, &active.progress);
    let fade = progress.fade();
    if let Ok(mut backdrop) = roots.get_mut(active.root) {
        backdrop.0 = Color::BLACK.with_alpha(spec.backdrop * fade);
    }
    for (mut bg, mut border) in &mut boxes {
        bg.0 = with_alpha(BOX_BG, fade);
        *border = BorderColor::all(with_alpha(theme.text.label_color, 0.6 * fade));
    }
    for mut color in &mut titles {
        color.0 = with_alpha(theme.text.header_color, fade);
    }

    let page = spec.lines.get(progress.page).map_or("", String::as_str);
    let (typed, rest) = split_at_chars(page, progress.visible_chars(spec));
    for (mut span, mut color) in &mut shown {
        if span.0 != typed {
            typed.clone_into(&mut span.0);
        }
        color.0 = with_alpha(theme.text.label_color, fade);
    }
    for mut span in &mut hidden {
        if span.0 != rest {
            rest.clone_into(&mut span.0);
        }
    }

    let blink = if progress.page_complete(spec) && progress.closing.is_none() {
        0.55 + 0.45 * (time.elapsed_secs() * 4.0).sin()
    } else {
        0.0
    };
    for mut color in &mut hints {
        color.0 = with_alpha(theme.text.label_color, blink * fade);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp, reason = "the compared values are exact")]
mod tests {
    use super::*;

    fn typewriter(lines: &[&str]) -> CrawlSpec {
        CrawlSpec {
            lines: lines.iter().map(ToString::to_string).collect(),
            style: CrawlStyle::Typewriter,
            speed: 10.0,
            ..CrawlSpec::default()
        }
    }

    /// Past the fade-in, so input is accepted.
    fn shown() -> CrawlProgress {
        CrawlProgress {
            elapsed: FADE_IN_SECS,
            ..CrawlProgress::default()
        }
    }

    #[test]
    fn splitting_respects_character_boundaries() {
        assert_eq!(split_at_chars("hello", 2), ("he", "llo"));
        assert_eq!(split_at_chars("hello", 9), ("hello", ""));
        assert_eq!(split_at_chars("åäö!", 2), ("åä", "ö!"));
        assert_eq!(split_at_chars("", 3), ("", ""));
    }

    #[test]
    fn a_typewriter_types_at_its_speed() {
        let spec = typewriter(&["abcdefghij"]);
        let mut p = CrawlProgress::default();
        p.tick(0.5, &spec);
        assert_eq!(p.visible_chars(&spec), 5);
        p.tick(0.6, &spec);
        assert_eq!(p.visible_chars(&spec), 10);
        assert!(p.page_complete(&spec));
    }

    #[test]
    fn advancing_completes_the_page_then_turns_it_then_closes() {
        let spec = typewriter(&["abcdefghij", "xy"]);
        let mut p = shown();
        p.advance(&spec);
        assert!(p.page_complete(&spec), "first press finishes typing");
        assert_eq!(p.page, 0);
        p.advance(&spec);
        assert_eq!(
            (p.page, p.visible_chars(&spec)),
            (1, 0),
            "second turns the page"
        );
        p.advance(&spec);
        p.advance(&spec);
        assert!(p.closing.is_some(), "past the last page it closes");
        assert!(!p.skipped);
    }

    #[test]
    fn input_during_the_fade_in_is_ignored() {
        let spec = typewriter(&["abc"]);
        let mut p = CrawlProgress::default();
        p.advance(&spec);
        assert_eq!(p.visible_chars(&spec), 0);
    }

    #[test]
    fn auto_advance_turns_a_finished_page() {
        let mut spec = typewriter(&["ab", "cd"]);
        spec.auto_advance = 1.0;
        let mut p = CrawlProgress::default();
        p.tick(0.3, &spec); // typed
        p.tick(0.5, &spec);
        assert_eq!(p.page, 0);
        p.tick(0.6, &spec);
        assert_eq!(p.page, 1);
    }

    #[test]
    fn a_scroll_closes_at_its_end_and_then_finishes_after_the_fade() {
        let spec = CrawlSpec {
            speed: 100.0,
            ..CrawlSpec::default()
        };
        let mut p = CrawlProgress::default();
        assert!(!p.tick(5.0, &spec), "no end known yet: keeps rolling");
        assert!(p.closing.is_none());
        p.scroll_end = Some(600.0);
        p.tick(1.0, &spec);
        assert!(p.closing.is_some());
        assert!(!p.tick(FADE_OUT_SECS / 2.0, &spec));
        assert!(p.tick(FADE_OUT_SECS, &spec));
    }

    #[test]
    fn a_skippable_scroll_can_be_cut_short_and_says_so() {
        let spec = CrawlSpec::default();
        let mut p = shown();
        p.advance(&spec);
        assert!(p.skipped && p.closing.is_some());

        let fixed = CrawlSpec {
            skippable: false,
            ..CrawlSpec::default()
        };
        let mut p = shown();
        p.advance(&fixed);
        assert!(p.closing.is_none());
    }

    #[test]
    fn fade_ramps_in_and_out() {
        let mut p = CrawlProgress::default();
        assert_eq!(p.fade(), 0.0);
        p.elapsed = FADE_IN_SECS;
        assert_eq!(p.fade(), 1.0);
        p.closing = Some(FADE_OUT_SECS);
        assert_eq!(p.fade(), 0.0);
    }

    #[test]
    fn a_flat_roll_is_a_straight_line() {
        assert_eq!(project(100.0, 800.0, 0.0), (700.0, 1.0));
        assert_eq!(exit_depth(800.0, 0.0), 800.0);
    }

    #[test]
    fn a_tilted_roll_shrinks_with_depth_and_exits_where_promised() {
        let h = 800.0;
        let (bottom, s0) = project(0.0, h, 1.0);
        assert_eq!((bottom, s0), (h, 1.0));
        let (mid_y, mid_s) = project(400.0, h, 1.0);
        assert!(
            mid_y < h - 300.0 && mid_y > 0.0,
            "compressed, still on screen"
        );
        assert!(mid_s < 1.0 && mid_s > 0.5);
        let (exit_y, _) = project(exit_depth(h, 1.0), h, 1.0);
        assert!(
            exit_y.abs() < 1e-2,
            "exit_depth lands on the top edge: {exit_y}"
        );
    }

    #[test]
    fn the_queue_reports_busy_and_modal() {
        let mut queue = CrawlQueue::default();
        assert!(!queue.is_busy());
        queue.push(CrawlSpec::default());
        assert!(queue.is_busy());
        assert!(queue.is_modal(), "the next in line counts");
    }
}
