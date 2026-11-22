# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Lint (the pedantic set in [lints.clippy] is kept in sync with aliens-vs-suburbia by hand)
cargo clippy --all-targets --all-features

# Test (--all-features so the serde-gated crawl code is covered too). Slow the first time:
# it links every example against full bevy.
cargo test --all-features
cargo test --test widgets                # one integration file (scenes | widgets | crawl)
cargo test follower_axis                 # tests whose name contains the filter

# Library alone, with the crate's own trimmed bevy features (see Constraints)
cargo check --lib

# Examples: one per UI challenge (see challenges.md), plus a few API demos
cargo run --example 1_game_menu           # 1: screens, slider/toggle/select, focus nav
cargo run --example 2_inventory           # 2: images, drag and drop, tooltips
cargo run --example 3_health_bar          # 3: WorldFollower + component bindings, 3D
cargo run --example 4_responsive_menu     # 4: nine-slice skinned buttons, breakpoints
cargo run --example 5_character_editor    # 5: 3D preview, choices, scroll, text input
cargo run --example 6_hud                 # 6: anchored corners, resource bindings
cargo run --example 7_bug_report          # 7: dropdown, text input, text area, toasts
cargo run --example 8_scoreboard          # 8: table, avatars, rebuild from ECS
cargo run --example 9_dark_light_theme    # 9: theme presets switched live
cargo run --example buttons             # variants, states, feathers side by side
cargo run --example bsn_layout          # small scene example
cargo run --example complex_example     # collapsibles, scrolling, runtime rebuild
cargo run --example basic_layout        # the OLD APIs, kept as the unmigrated reference

# Regenerate the example images in assets/ (stdlib-only python)
python3 tools/make_assets.py assets
```

`bacon.toml` wraps all of the above as jobs (nightly comes from `../rust-toolchain.toml`;
caching is kache, set up in `~/.cargo/config.toml`; compiler flags such as `-Z threads`
live in `../.cargo/config.toml`, never in bacon or the shell env,
so every cargo invocation shares one cache):
`bacon` (clippy-all by default), or press `y` clippy, `t` test, `e` example,
`r` check-all-examples; `bacon ex -- <name>` runs one example by name. Keep every job on
the same flags — a job with different ones re-fingerprints the whole bevy tree.

Tests: unit tests in `src/systems.rs` (`follower_axis`) and `src/crawl.rs`; headless
integration tests in `tests/scenes.rs` (entity trees, `#Name` resolution, tokens),
`tests/widgets.rs` (variants/states, presets, bindings, controls, screens, dialogs, toasts)
and `tests/crawl.rs`. They use `MinimalPlugins + AssetPlugin + ScenePlugin` plus
`tokens::ThemeTokensPlugin` (and `controls::ControlsPlugin` / `overlay::OverlayPlugin` as
needed) — no window or renderer, so `LavaUiPlugin` itself (picking, input, focus) is not
added. Focus movement, layout and looks are validated by running the examples. The README
code block is compiled as a doctest (`ReadmeDoctests` in `lib.rs`).

## Constraints

- **Panicking is a lint error.** `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`,
  `string_slice`, `todo`, `unreachable` are `deny` in `[lints.clippy]`. Use `let-else`,
  `.get()`, `Option` combinators. Test files opt out with a file-level
  `#![allow(..., reason = "...")]`.
- **The library's bevy is trimmed** (`default-features = false`, only `ui`, `bevy_feathers`,
  `bevy_scene`), while `[dev-dependencies]` pulls full bevy for examples and tests. So code
  using a bevy module outside those features compiles under `cargo test`/examples but breaks
  `cargo check --lib` and downstream builds. `ui` already brings `bevy_state`,
  `bevy_input_focus`, `bevy_ui_widgets`, picking and text editing. Don't widen the feature
  list casually: the Cargo.toml comment explains the Android activity conflict with the
  `civilization` game.
- **Downstream consumers** pull this crate in as a `lava_ui_builder/` subdirectory (a git
  submodule): `../aliens-vs-suburbia` (with `serde`) and `../civilization` (as a workspace
  member). Public API changes need a matching entry in `migration-guide.md`. They build
  `InteractionPalette { none, hovered, pressed }` literally (do not add fields to it) and
  themes with `..LavaTheme::default()` / `..t.clone()` (adding theme fields is fine).
  About 30 downstream files use `UIBuilder`: it and the bundle API are frozen, not removed.

## Architecture

`lava_ui_builder` is a game UI kit for Bevy 0.20. Plan and rationale: `roadmap.md`.
Three APIs live here; the **scene API** is the one to use and the only one that gets new
widgets. `bsn-migration.md` explains how the APIs relate; `migration-guide.md` is for
updating a *downstream* project.

**The one rule that bites:** `apply_theme_tokens` owns `TextColor`, `BackgroundColor`,
`BorderColor`, `TextFont` and `InteractionPalette` on any entity carrying a token or a
`Variant`, and rewrites them every frame. Never write those components on such an entity;
patch the token / variant instead, or move a state marker (`Selected`, `Checked`,
`InteractionDisabled`). Use the explicit widgets (`text`, `button_colored`,
`progress_bar`) only when the colour is content rather than theme.

### Module map
| module | what |
|---|---|
| `theme.rs` | `LavaTheme` + sub-themes (`button` with variant `ButtonColors`, `text`, `meter`, `surface`, `controls`), presets `dark/light/from_accent`, `with_font` |
| `tokens.rs` | `ColorToken`/`FontToken`, the `Themed*` components, `Variant` + `VariantText`, `apply_theme_tokens` (in `ThemeSystems`), `ThemeTokensPlugin` |
| `scenes/` | scene functions: `layout.rs`, `text.rs`, `buttons.rs` (incl. `focusable()`), `meters.rs`; `mod.rs` re-exports everything below so all widgets are `scenes::*` |
| `controls.rs` | slider, checkbox/toggle, choice (radio group of buttons), cycle, select (dropdown), text_input/text_area — themed looks on `bevy::ui_widgets` |
| `bind.rs` | `bind_*` scene fns + `BindAppExt::{bind_resource, bind_component}` |
| `screens.rs` | `ScreenAppExt::{add_screen, screen_back}`, `GoTo<S>` / `go_to`, `Back` |
| `focus.rs` | Tab (own handler, not Bevy's plugin) and arrow/D-pad/stick navigation under `UiNavigation`, gamepad A/B, `NavigateBack`, focus ring, scroll-into-view, `UiNavigation` settings |
| `overlay.rs` | `Tooltip`, `Toast` (an `Event`), `confirm` dialogs + `DialogResult` |
| `media.rs` | `image`, `icon`, `nine_slice`, `skinned_button` (`PaletteTintsImage`) |
| `drag.rs` | `draggable`, `drop_target`, `Dropped` |
| `responsive.rs` | `Responsive { breakpoint, narrow: NarrowLayout }`, `stack_below`, `full_width_below` |
| `components.rs` | `InteractionPalette`, collapsible parts, `ProgressBar`, `WorldFollower` |
| `systems.rs` | palette application (incl. image tint), scrolling, collapsibles, progress bars, world followers |
| `bundles.rs`, `builder.rs`, `button_builder.rs` | the frozen bundle-function and `UIBuilder` APIs |
| `crawl.rs` | story crawls (`CrawlPlugin`, separate from `LavaUiPlugin`) |

`lib.rs` holds module declarations, root re-exports (downstream paths like
`lava_ui_builder::TextTheme` must keep working), the `prelude`, the `UIBuilder` struct
definitions and `LavaUiPlugin`, which adds the token, controls, focus, overlay and drag
plugins. Bindings and screens are generic, so they are registered per type by the app.

### Writing scenes
- calling a scene function inside `bsn!` needs `@`: `Children [ @scenes::label("a") -- @scenes::label("b") HealthText ]`.
  Without `@` it fails with "`impl Scene: Template` is not satisfied"
- Bevy 0.20 BSN separates entities with `--`; components on one entity are just listed.
  Commas between entities, `( .. )` around them, and `bsn_list![..]` / `bsn_list!(..)`
  (instead of `bsn_list! {..}`) still compile but emit deprecation warnings. The tree is
  clean of them; keep it that way
- a `Scene` is `'static`, so it cannot borrow: copy values into locals before the `bsn!`
- anything that is not BSN value syntax needs braces — `Text({text})`, `{Some(x)}`
- a whole component *value* goes in `~{value}` (how the `bind_*` and `go_to` fns insert
  generic components); an optional scene goes in `@{option}`; `Option<impl Scene>` and
  tuples of scenes are scenes too, so `(bsn!{..}, cond.then_some(bsn!{ Selected }))` adds
  a marker conditionally
- an optional child is `Option<impl SceneList>`: `cond.then(|| bsn_list! {..})`; a
  `Children [..]` slot takes a list, so a single scene goes in as `vec![scene]`
- a `Vec` of children must be one type: give cells a common shape (see `8_scoreboard.rs`)
- generic components need a turbofish in BSN (`GoTo::<S>(..)`) and `S: Unpin`; hide that
  behind a scene fn (`go_to(state)`)
- `on(..)` observers must be `Clone`: capture `Copy` data or clone into the closure;
  bindings hold plain `fn` pointers for the same reason
- a parameter named `on` shadows BSN's `on(..)` — don't
- interactive widgets include `@focusable()` (TabIndex + `AutoDirectionalNavigation`)
- every widget is themed: a new colour role is a `LavaTheme` field (in a sub-struct, with a
  `Default`, also set in `light()`) plus a `ColorToken` variant resolved in
  `ColorToken::color` — not a `Color` parameter. Check `bevy::ui_widgets` /
  `bevy::feathers` for existing behaviour before hand-rolling (0.20 has slider, select,
  menu, text input, checkbox, radio, scroll area, tabs, dialog; no meter)

Runtime rebuilds (e.g. a list whose contents change) go through
`scenes::replace_children(commands, parent, list)`. The new children spawn in the
`SpawnScene` schedule, after `Update`, so the system doing the rebuild cannot query them in
the same run — and an entity spawned from a scene at `Startup` may not exist yet in the
first `Update` (see the `Added` check in `8_scoreboard.rs`).

### Bevy 0.20 API notes (things that differ from older docs)
- picking events are concrete types: `On<PointerPress>`, `On<PointerDragStart>`, …
  (not `Pointer<Press>`); global observers see every bubbling hop — compare
  `ev.entity` with `ev.original_event_target()`
- resources are components: a generic `ResMut<R>` needs `R: Resource + Component<Mutability = Mutable>`;
  `NextState<S>::set` needs `S: FreelyMutableState`
- `Query::iter_many` yields `Result`s
- `SliderValue` is immutable: insert a new one to change it
- radio groups consume all four arrow keys when the *group* is focused, which is why
  `choice` makes each option focusable instead

### Legacy notes
- `apply_interaction_palette` reads `Hovered`/`Pressed` (the `bevy_ui_widgets` state) and
  falls back to the legacy `Interaction`; use `bevy::ui_widgets::Button` + `Hovered`
- `toggle_collapsible_on_activate` is a global observer on `Activate`
- feathers helpers: `bevy_feathers` is an unconditional dependency (no feature);
  `.text()`, `.font_size()`, `.text_color()` are no-ops on `feathers_button_with_overrides`
- components used inside `bsn!` need `Default + Clone`, or `#[derive(FromTemplate)]` when
  they hold an `Entity` (resolved from a `#Name`) or an asset handle
- crawls: `CrawlPlugin` owns a `CrawlQueue`; text colours are plain `TextColor` (the crawl
  fades them), not tokens; `CrawlProgress` is the unit-tested timing core
