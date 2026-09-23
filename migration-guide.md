# Migrating a project that uses `lava_ui_builder`

You are updating a **consumer** codebase (a game or tool) after `lava_ui_builder` gained a
BSN scene API. This guide is the operational counterpart to `bsn-migration.md`, which
documents the library side and the reasoning; read that only if you need the *why*.

**Read this section before changing anything.**

The update is **incremental and mostly optional**. The bundle-function API (`ui_root`,
`header`, `label`, `themed_button`, `progress_bar`, `spawn_list_item`) and the imperative
`UIBuilder` both still exist, still compile, and are not deprecated. A consumer can bump
the dependency, apply the short list in §1, and stop there — that is a legitimate
end state, and for a large UI it is the *recommended* first commit.

Do **not** rewrite working UI code into scenes just because the scene API exists. Migrate
a screen when you are already editing it, or when it has a problem scenes solve (runtime
rebuilds, theme switching, deeply nested `with_child` closures). Mixing the two APIs in
one tree is supported and expected — §3 has the bridge.

---

## 1. Mandatory: what breaks on the version bump

Four things. Two are compile errors (loud), two are behavioural (silent — these are the
dangerous ones).

### 1.1 `systems::handle_collapse_toggle` no longer exists — compile error

Collapse toggling moved from a polling system to a global observer on `Activate`, so it
works regardless of which API spawned the button and responds to ENTER/SPACE.

```bash
grep -rn "handle_collapse_toggle" src/
```

If found, delete the registration. `LavaUiPlugin` registers the replacement
(`systems::toggle_collapsible_on_activate`) for you.

```rust
// before
.add_systems(Update, (lava_ui_builder::systems::handle_collapse_toggle, ...))
// after — nothing; LavaUiPlugin does it
```

### 1.2 `WorldFollower` no longer derives `Clone` — compile error if you cloned it

It derives `FromTemplate` instead, so it can take an `Entity` from a `#Name` reference in
`bsn!`. Struct-literal construction (`WorldFollower { target, offset }`) is **unaffected**;
only `.clone()` breaks.

```bash
grep -rn "WorldFollower" src/ | grep -i clone
```

Same change applies to `CollapseToggleButton` and `CollapsibleContent`. If you genuinely
need a clone, copy the fields — both are `Copy`-able data (`Entity`, `Vec2`).

### 1.3 SILENT: never write `BackgroundColor` on an entity that has an `InteractionPalette`

This is the one to be careful about. `systems::apply_interaction_palette` now runs every
frame and **owns** `BackgroundColor` on any entity carrying an `InteractionPalette`. A
direct write lasts exactly one frame and is then reverted — no error, no warning.

Find the risk:

```bash
# the reliable signal: anything that can mutate a background colour after spawn
grep -rn "mut BackgroundColor" src/
```

No hits means this hazard does not apply to the project at all — a spawn-time
`BackgroundColor(..)` is fine and needs no change.

Then check whether those entities also get an `InteractionPalette`. **Spawning** with both
is fine and normal (`(BackgroundColor(bg), InteractionPalette { none: bg, .. })`) — the
palette takes over from there. The problem is only a *later* write, typically a selection
highlight:

```rust
// before — stops working
for (btn, mut bg) in &mut buttons {
    bg.0 = if selected { GREEN } else { BLUE };
}

// after — swap the palette; the palette system paints from it
for (btn, mut palette) in &mut buttons {
    *palette = if selected { SELECTED_PALETTE } else { UNSELECTED_PALETTE };
}
```

Where `SELECTED_PALETTE` is a `const InteractionPalette { none, hovered, pressed }`.
`examples/game_menu.rs` in the library is a worked example of exactly this change.

If an entity must keep a hand-painted background, remove its `InteractionPalette`.

### 1.4 The `feathers` Cargo feature was removed

It gated nothing (`bevy_feathers` is an unconditional dependency), so `--features feathers`
was a no-op. It is now an error.

```bash
grep -rn 'lava_ui_builder.*feathers' Cargo.toml
```

Remove `features = ["feathers"]` from the dependency line if present. The feathers helpers
(`feathers_button`, `feathers_button_primary`, …) still compile unconditionally.

---

## 2. What gets better with no work from you

Mention these when reporting, and **verify them visually** — they are behaviour changes
even though they are fixes:

* **Hover and press colours start working on buttons that previously looked dead.**
  `apply_interaction_palette` used to read the legacy `Interaction` component, which
  `bevy::ui_widgets::Button` does not carry. Any entity built with `ui_widgets::Button` +
  `Hovered` + `InteractionPalette` had an inert palette. It now reads `Hovered`/`Pressed`.
  If a UI looked intentionally flat, it may now animate on hover — that is the fix, not a
  regression, but a human should confirm the colours were chosen deliberately.
* **Legacy `Interaction` still works.** The palette prefers `Hovered`/`Pressed` and falls
  back to `Interaction`, so entities built with `bevy_ui::widget::Button` keep working.
* **Collapsible toggles respond to ENTER/SPACE** when focused, and work regardless of how
  they were spawned.

---

## 3. Optional: moving a screen to the scene API

### 3.1 Prerequisites

* `LavaUiPlugin` **must** be added, or theme tokens never resolve and every themed widget
  renders with default colours. (It was already required for collapsibles and scrolling.)
* `bsn!` comes from `bevy::prelude` and needs bevy's `bevy_scene` feature. The `ui` feature
  pulls it in transitively, so most consumers already have it. If bevy is depended on with
  `default-features = false` and a hand-written feature list that does not include `ui`,
  add `"bevy_scene"` explicitly.
* `ScenePlugin` ships inside `DefaultPlugins`. A custom plugin set needs it added.

### 3.2 The bridge — migrate a subtree, not a file

Do not rewrite a whole screen in one go. `UIBuilder` can hand a subtree to a scene:

```rust
ui.with_child(|c| {
    c.apply_scene(scenes::button("Play"));      // patch a scene onto this entity
});
ui.scene_child(scenes::label("hello"));          // spawn a scene as a child
ui.scene_children(bsn_list![ /* … */ ]);         // … or a whole list
```

And a scene tree can be rebuilt in place, replacing
`UIBuilder::start_from_entity(.., clear_children: true)`:

```rust
scenes::replace_children(&mut commands, list_root, bsn_list![
    scenes::list_item("one", false),
    scenes::list_item("two", true),
]);
```

The parent entity survives, so stored `Entity` ids, marker queries and `ScrollPosition` all
keep working. Note the new children appear in the `SpawnScene` schedule (after `Update` in
the same frame), so a system cannot rebuild a list and query the result in the same run.

### 3.3 Pattern translation

| Imperative / bundle | Scene |
| --- | --- |
| `UIBuilder::new(commands, theme)` + `set_node` | `commands.spawn_scene(bsn! { … })` |
| `.with_child(\|c\| { … })` nesting | `Children [ ( … ) ]` |
| `.foreach_child(items, …)` | `Children [ {items.map(…).collect::<Vec<_>>()} ]` |
| `if cond { ui.with_child(…) }` | `Children [ {cond.then(\|\| bsn_list![…])} ]` |
| `.add_button_observe(text, cfg, handler)` | `scenes::button(text) on(handler)` |
| `ButtonBuilder::size_px(w, h)` | `Node { width: px(w), height: px(h) }` patch |
| `.with_text(t, Some(TextStyle::size_color(s, c)))` | `scenes::text(t, s, c)` |
| themed label/header | `scenes::label(t)` / `scenes::header(t)` (no theme argument) |
| `.add_row(…)` / `.add_column(…)` | `scenes::row(gap)` / `scenes::column(gap)` + `Children` |
| `.scrollable_list(…)` | `scenes::scrollable_list(gap)` |
| `.list_item(name, selected, h)` | `scenes::list_item(name, selected) on(h)` |
| `.with_collapsible(label, collapsed, …)` | `scenes::collapsible(label, collapsed, bsn_list![…])` |
| `progress_bar(v, w, h, fill, bg)` bundle | `scenes::progress_bar(v, w, h, fill, bg)` |
| a startup system building UI from queries | same system, ending in `commands.spawn_scene(…)` |
| a UI with no world data | `fn ui() -> impl Scene` + `.add_systems(Startup, ui.spawn())` |

### 3.4 Themed vs explicit widgets — pick the right one

| | Themed | Explicit |
| --- | --- | --- |
| widgets | `label`, `header`, `button`, `collapsible` | `text`, `button_colored`, `list_item`, `icon_button`, `delete_button` |
| colour source | `LavaTheme` via tokens | concrete `Color` arguments |
| follows a theme swap | yes | no |
| can be recoloured by patching | **no** | yes |

The "no" is mechanical, not stylistic: `tokens::apply_theme_tokens` rewrites `TextColor`
and `InteractionPalette` every frame for entities carrying tokens, so a patched colour
survives one frame. To recolour a themed widget, patch its **token**:

```rust
scenes::label("danger") ThemedTextColor(ColorToken::HeaderText)   // works
scenes::label("danger") TextColor(Color::RED)                     // reverts next frame
scenes::text("danger", 16.0, Color::RED)                          // works — untokened
```

### 3.5 The five things that will bite you

1. **`on(..)` needs a `Clone` observer system.** A closure capturing a `String` is not
   `Clone`-friendly in practice; capture `&'static str` or `Copy` data, or build the
   closure in a factory function (`fn press(what: &'static str) -> impl Fn(..) + Clone`).
2. **An optional child is `Option<impl SceneList>`, not `Option<impl Scene>`.** A
   `Children [..]` slot interpolates a *list*: `cond.then(|| bsn_list![..])`. Getting this
   wrong produces a confusing `SceneList is not implemented for impl Scene` error.
3. **Components used in `bsn!` need `Default + Clone`**, or `#[derive(FromTemplate)]` when
   they hold an `Entity` or an asset handle. Marker components need
   `#[derive(Component, Default, Clone)]`. You cannot derive both `Default` and
   `FromTemplate`.
4. **Braces are for component values, not function arguments.** `Text({text})` needs them;
   `scenes::text(format!("{x}"), 14.0, colour)` does not. Rule: a scene-function call is
   ordinary Rust; a component's field or tuple value is BSN syntax. Bare enum variants and
   associated consts (`Pickable::IGNORE`) work without braces.
5. **`TextFont.font` is a `FontSourceTemplate` inside `bsn!`**, not a `Handle<Font>`. Use
   the theme (`ThemedFont(FontToken::Label)`) rather than passing a handle. If you truly
   need a resolved handle: `FontSourceTemplate::Handle(HandleTemplate::Handle(h))`.

### 3.6 Delete local copies of library systems

Consumers accumulated their own versions of systems that live in the library. The library
ones are now the fixed ones; local copies of the palette system are the *broken*
`Interaction`-based version.

```bash
grep -rn "fn .*interaction_palette\|fn .*scroll_input\|fn .*collapse_toggle\|fn .*collapsible_visibility" src/
```

Delete any hit and rely on `LavaUiPlugin`.

---

## 4. Verification checklist

Run in this order; stop at the first failure.

```bash
cargo clippy --all-targets            # compile errors from §1.1, §1.2, §1.4
cargo test
cargo run --bin <the app>             # or the relevant example
```

Then check by hand, because none of the above catches §1.3:

* every selection highlight still highlights (list rows, tab bars, quality/difficulty
  pickers — anything that used to write `BackgroundColor` from a system);
* buttons that should react on hover do, and buttons that should *not* look interactive
  still don't;
* collapsible sections open and close, including via keyboard;
* if the project switches themes at runtime, that it still works — and if it rebuilds the
  UI to do so, note that tokens make the rebuild unnecessary (see
  `examples/dark_light_theme.rs` in the library).

## 5. Reference

```rust
// scenes:: — every one returns `impl Scene`
ui_root()                                   header(text)              label(text)
text(content, size, color)                  section_label(content)
button(content)                             button_colored(content, none, hovered, pressed)
progress_bar(value, width, height, fill, bg)
collapsible(label, collapsed, content: impl SceneList)
row(column_gap)   column(row_gap)   panel(padding, row_gap)   grid(cols, gap)
centered()        spacer()          side_panel(width, bg)
scrollable_list(gap)                scrollable_list_bounded(gap, max_height)
list_item(name, selected)           icon_button(glyph, size)   delete_button()
replace_children(&mut commands, parent, children: impl SceneList)   // not a Scene

// tokens:: — components resolved against LavaTheme by LavaUiPlugin
ThemedBackground(ColorToken)   ThemedTextColor(ColorToken)   ThemedBorderColor(ColorToken)
ThemedFont(FontToken)          ThemedPalette { none, hovered, pressed }

ColorToken:: ButtonBg | ButtonBgHovered | ButtonBgPressed | ButtonText | ButtonBorder
           | CollapsibleBg | CollapsibleBgHovered | CollapsibleBgPressed
           | HeaderText | LabelText | PanelBg | PanelBorder
FontToken::  Header | Label | Button
```

Worked examples in the library, roughly in order of complexity: `bsn_layout` (smallest),
`buttons` (observers, patching, `@FeathersButton`), `scoreboard` (data-driven rows),
`game_menu` (panels, selection-as-palette), `dark_light_theme` (live theming),
`hud` (corner anchoring, marker-driven systems), `inventory` (inline drag/drop observers),
`complex_example` (collapsibles, scrolling, `replace_children`). `basic_layout` is
deliberately left on the old APIs as the unmigrated reference.
