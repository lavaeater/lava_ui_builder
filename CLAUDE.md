# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Lint (the pedantic set in [lints.clippy] is kept in sync with aliens-vs-suburbia by hand)
cargo clippy --all-targets --all-features

# Test
cargo test

# Run an example (all are on the scene API except basic_layout)
cargo run --example bsn_layout          # smallest scene example; start here
cargo run --example dark_light_theme    # live theme switching via tokens
cargo run --example inventory           # inline on(..) observers, drag and drop
cargo run --example complex_example     # collapsibles, scrolling, runtime rebuild
cargo run --example basic_layout        # the OLD APIs, kept as the unmigrated reference
```

`bacon.toml` wraps all of the above as jobs on nightly with cranelift + sccache:
`bacon` (clippy-all by default), or press `y` clippy, `t` test, `e` example,
`r` check-all-examples; `bacon ex -- <name>` runs one example by name. Keep every job on
the same flags — a job with different ones re-fingerprints the whole bevy tree.

Tests: `src/systems.rs` has unit tests for `follower_axis`; `tests/scenes.rs` has headless
tests that spawn each BSN scene and assert the resulting entity tree. Visual behaviour is
still validated by running the examples.

## Architecture

`lava_ui_builder` is a Bevy 0.19 UI library. It provides three APIs; the scene API is the
one to use for new code. See `bsn-migration.md` for how they relate and what moved.

**The one rule that bites:** `apply_theme_tokens` owns `TextColor` and
`InteractionPalette` on any entity carrying a token, and rewrites them every frame. Never
write `BackgroundColor` on an entity that has an `InteractionPalette` — swap the palette
instead. Themed widgets (`label`, `header`, `button`, `collapsible`) cannot be recoloured
by patching; use the explicit ones (`text`, `button_colored`, `list_item`, `icon_button`)
when the colour is content rather than theme.

### Scene API (`src/scenes.rs`) — preferred
Free functions (`ui_root`, `header`, `label`, `button`, `progress_bar`, `collapsible`)
that return `impl Scene`, built with the `bsn!` macro. They compose inside `Children [..]`
and callers override single fields by *patching* (`scenes::button("Play") Node { width: px(220) }`)
rather than by passing options. Widgets take **no theme argument** — colors and fonts come
from `src/tokens.rs` (see below).

Three rules when writing them:
- a `Scene` is `'static`, so it cannot borrow: copy values into locals before the `bsn!`
- anything that is not BSN value syntax needs braces — `Text({text})`, not `Text(text.into())`
- an optional child is `Option<impl SceneList>`: `cond.then(|| bsn_list![..])`, *not*
  `bsn!{..}`, because a `Children [..]` slot interpolates a list

### Theme tokens (`src/tokens.rs`)
`ColorToken` / `FontToken` are enums naming the fields of `LavaTheme`. Widgets carry
`ThemedBackground` / `ThemedTextColor` / `ThemedBorderColor` / `ThemedFont` /
`ThemedPalette`, and `apply_theme_tokens` resolves them against the resource every frame
(guarded by equality checks). Swapping the `LavaTheme` resource repaints live UI — no
rebuild. Patching a bare `TextColor` over a themed widget will **not** stick; patch the
token instead. Sizes are not tokens: patch `Node`.

### Bundle-function API (`src/lib.rs`)
Free functions (`ui_root`, `header`, `label`, `themed_button`, etc.) that return
`impl Bundle` and compose with Bevy's `children![]` macro. Being superseded by the scene
API; still used by most examples.

### Imperative builder API (`src/builder.rs`, `src/button_builder.rs`)
`UIBuilder` — an owned struct holding a `Commands` reference, a `current_entity`, a `parent_stack` (VecDeque), and a `LavaTheme`. Navigation is done with `.child()` / `.parent()`. Buttons are created via `.add_themed_button()` / `.add_button_observe()`. Returns `ButtonBuilder` for further configuration inside a closure.

### Theme system
`LavaTheme` is a Bevy `Resource` containing `ButtonTheme` and `TextTheme`. All hardcoded colors/sizes in the builder methods come from the theme. `ui_width` / `ui_height` drive `adapt_ui_scale`. Insert `LavaTheme` as a resource before building UI; if absent, `UIBuilder::new()` falls back to `Default`.

### Key types
- `InteractionPalette` — component with `none/hovered/pressed` colors; applied by `systems.rs`.
  Scene widgets fill it from a `ThemedPalette` rather than setting it directly
- `TextStyle` — optional style overrides (size, color, font, justify, line_break) passed to `with_text`; use `TextStyle::size()`, `::color()`, `::size_color()` convenience constructors
- `Collapsible` / `CollapsibleContent` / `CollapseToggleButton` — components for collapsible sections; the toggle system lives in `systems.rs`
- feathers helpers: `bevy_feathers` is an unconditional dependency, so
  `feathers_button_with_overrides()` and friends always compile — there is no feature to
  enable (the no-op `feathers` feature was removed). `.text()`, `.font_size()` and
  `.text_color()` are no-ops on that builder. In scenes, use `@FeathersButton` directly;
  see `examples/buttons.rs`.
- Components used inside `bsn!` need `Default + Clone`, or `#[derive(FromTemplate)]` when
  they hold an `Entity` (resolved from a `#Name`) or an asset handle

### `systems.rs`
`LavaUiPlugin` (defined in `lib.rs`) registers:
- `apply_interaction_palette` — applies `InteractionPalette` colors. Reads `Hovered` and
  `Pressed` (the `bevy_ui_widgets` state), **not** the legacy `Interaction` component:
  `ui_widgets::Button` does not require `Interaction`, so a palette driven by it silently
  never fired. Use `bevy::ui_widgets::Button` + `Hovered` on anything with a palette.
- `apply_theme_tokens` (in `tokens.rs`) — resolves theme tokens; `.chain()`-ed to run
  *before* `apply_interaction_palette`, since tokens produce the palette
- `toggle_collapsible_on_activate` — a **global observer** on `Activate`, not a system,
  so toggles work however they were spawned and respond to ENTER/SPACE
- `update_collapsible_visibility` — shows/hides `CollapsibleContent`, updates the arrow
- `handle_scroll_input`, `sync_progress_bars`, `world_follower_system`
- `adapt_ui_scale` (in `lib.rs`) — rescales UI on window resize (uses `LavaTheme.ui_width`)
