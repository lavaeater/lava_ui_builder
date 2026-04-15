# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Lint
cargo clippy

# Run an example
cargo run --example basic_layout
cargo run --example buttons
cargo run --example complex_example

# Run with the feathers feature flag
cargo run --example basic_layout --features feathers
```

There are no tests in this crate — validation is done by running examples.

## Architecture

`lava_ui_builder` is a fluent Bevy 0.18 UI builder library. It provides two complementary APIs:

### Bundle-function API (`src/lib.rs`)
Free functions (`ui_root`, `header`, `label`, `themed_button`, etc.) that return `impl Bundle` and compose with Bevy's `children![]` macro. This is the preferred, idiomatic approach.

### Imperative builder API (`src/builder.rs`, `src/button_builder.rs`)
`UIBuilder` — an owned struct holding a `Commands` reference, a `current_entity`, a `parent_stack` (VecDeque), and a `LavaTheme`. Navigation is done with `.child()` / `.parent()`. Buttons are created via `.add_themed_button()` / `.add_button_observe()`. Returns `ButtonBuilder` for further configuration inside a closure.

### Theme system
`LavaTheme` is a Bevy `Resource` containing `ButtonTheme` and `TextTheme`. All hardcoded colors/sizes in the builder methods come from the theme. `ui_width` / `ui_height` drive `adapt_ui_scale`. Insert `LavaTheme` as a resource before building UI; if absent, `UIBuilder::new()` falls back to `Default`.

### Key types
- `InteractionPalette` — component with `none/hovered/pressed` colors; applied by `systems.rs`
- `TextStyle` — optional style overrides (size, color, font, justify, line_break) passed to `with_text`; use `TextStyle::size()`, `::color()`, `::size_color()` convenience constructors
- `Collapsible` / `CollapsibleContent` / `CollapseToggleButton` — components for collapsible sections; the toggle system lives in `systems.rs`
- `feathers` feature: enables `feathers_button_with_overrides()` which wraps Bevy's experimental feathers button; `.text()`, `.font_size()`, `.text_color()` are no-ops on that builder

### `systems.rs`
Exposes `LavaUiBuilderPlugin` which registers:
- `interaction_palette_system` — applies `InteractionPalette` colors on hover/press
- `collapsible_toggle_system` — shows/hides `CollapsibleContent` on `CollapseToggleButton` clicks
- `adapt_ui_scale` — scales UI nodes when the window is resized (uses `LavaTheme.ui_width`)
