# Roadmap

Where lava_ui_builder stands, what it is missing for "a game UI in very few lines that
is still easy to restyle", and the plan. Written 2026-11-19; the
[status log](#status-log) at the bottom records what has actually been done since.

## Is the concept worth it?

Yes. Bevy 0.20 ships two UI layers:

- `bevy_ui_widgets`: headless widgets (behaviour, no look).
- `bevy_feathers`: an editor-styled widget kit with a fixed look.

Nobody ships a **game UI kit with a swappable theme** on top of BSN. lava_ui_builder can
own that niche. The token-based theme is the strongest part and should stay the centre of
the design.

The risk is Bevy growing into the same space. The defence is to stay a *thin, opinionated
layer* over Bevy's own pieces:

- reuse `ui_widgets` for behaviour, and add looks plus game-shaped conveniences on top;
- never build a parallel framework.

## What the examples showed (before this work)

| example    | lines | raw `Color::` | sync systems | marker types | images |
|------------|------:|--------------:|-------------:|-------------:|-------:|
| game_menu  |   330 |            21 |            3 |            5 |      0 |
| hud        |   324 |            20 |            4 |            4 |      0 |
| health_bar |   289 |             7 |            1 |            1 |      0 |
| inventory  |   229 |            13 |            0 |            3 |      0 |
| scoreboard |   181 |            14 |            0 |            0 |      0 |

Three things stand out.

1. **Users fall off the theme almost immediately.**
   - The theme knows "button" and "label". Games need roles: danger, secondary, surface,
     muted, selected, disabled, focus.
   - So every example reached for raw colours: `button_colored`, `text(.., Color)`, and
     hand-written `BackgroundColor`. A theme swap then leaves most of the screen unchanged.
     `game_menu`'s panel did not repaint at all.
2. **About a quarter of each example is glue.**
   - `hud` has four copies of `if !state.is_changed() { return } … **t = format!(..)`.
   - `game_menu` spends three marker types and a three-way `Without<>` filter on showing
     one panel at a time.
3. **The "done" challenges were only partly met.**
   - No example used a single image: no item icons, avatars or minimap.
   - Challenge 1 had no slider, no dropdown and no keyboard or controller focus.

## Design principles (kept)

- **Widgets are scene functions returning `impl Scene`.** Callers customise by *patching*
  (`@scenes::button("Play") Node { width: px(260) }`), not by option structs.
- **No theme argument.** Colours and fonts are tokens resolved every frame against
  `LavaTheme`, so a theme swap repaints live UI without a rebuild.
- **Sizes are layout, not theme.** Patch `Node`. Sizes can't be tokens: a token system
  would overwrite the caller's patch every frame, which defeats patching.
- **Reuse Bevy's widgets for behaviour.** Slider, checkbox, radio, text input and focus
  come from `bevy_ui_widgets` / `bevy_input_focus`. lava adds the look and the wiring.
- **Downstream compatibility.**
  - aliens-vs-suburbia and civilization build `InteractionPalette { none, hovered, pressed }`
    and `TextTheme { .., ..t.clone() }` literally, so those keep their fields.
  - New theme fields are fine, because themes are built with `..LavaTheme::default()`.
  - The old bundle and builder APIs stay working (about 30 downstream files use
    `UIBuilder`). They are frozen, not deleted.

## The plan

### 1. Semantic theme roles, variants and states

This is what removes the raw `Color::` lines.

- **Roles.** New `LavaTheme` sub-structs:
  - `surface`: panel, raised, overlay scrim, tooltip, muted text, accent, success,
    warning, danger;
  - `controls`: track, fill, thumb, check, input, cursor, selection, focus ring.
  Each colour gets a `ColorToken`.
- **Variants.** A `Variant` component (`Primary`, `Secondary`, `Danger`, `Success`,
  `Ghost`) on any lava button: `@scenes::button("Quit") Variant::Danger`. The palette
  *and* the caption colour follow it, and `button_colored` becomes rare.
- **States.** The palette also follows Bevy's own state markers: `Checked` / `Selected`
  get the theme's "selected" colours, and `InteractionDisabled` the "disabled" ones. A
  selected quality button is then one marker, not a palette swap in a system.
- **Presets.**
  - `LavaTheme::dark()` and `LavaTheme::light()`;
  - `LavaTheme::from_accent(Color)`, which derives hover and press shades so a whole
    theme is one colour;
  - fonts, sizes and shades can still be overridden field by field.

### 2. Data binding

This is the biggest line saver. Text, meters and visibility are bound to a resource or to
a component on another entity:

```rust
@scenes::label("") BindText::<Hud>::new(|h| format!("{} / {}", h.ammo, h.ammo_max))
@scenes::meter(1.0, ..) BindMeter::<Health>::to(character, |h| h.current / h.max)
```

Registration is one line per source type, `app.bind_resource::<Hud>()`. Bindings only
write when the source changed, and their first write happens on spawn.

### 3. Screens and navigation

`app.add_screen(MenuScreen::Audio, audio_panel)` spawns the screen on `OnEnter` and
despawns it on exit, using Bevy `States`. `on(goto(MenuScreen::Audio))` is a ready-made
observer. Escape or gamepad B goes back through a `ScreenBack` resource. No marker types
and no show/hide systems are needed.

### 4. Themed controls on Bevy's headless widgets, plus focus

- **Controls:** `slider`, `checkbox`, `toggle`, `radio_group` and `choice`, `cycle`
  (`< Medium >`, gamepad-friendly), `select` (dropdown), and `text_input` (single-line and
  multi-line). Each emits Bevy's `ValueChange<T>` and is themed through tokens.
- **Focus:** every lava control can take focus (`TabIndex` + `AutoDirectionalNavigation`).
  A themed focus ring is drawn as an `Outline`. Arrow keys, D-pad and the left stick move
  focus, and gamepad A activates.

### 5. Images, nine-slice and responsive layout

- **Images:** `scenes::image(path)`, `scenes::icon(path, size)`, and `scenes::nine_slice`
  for skinned panels and buttons using Bevy's `NodeImageMode::Sliced`.
- **Responsive layout:** a `Responsive` component swaps between a wide and a narrow
  `Node` at a breakpoint.

### 6. Small game widgets

- `Tooltip("Diamond sword")` on any entity;
- toasts (`commands.trigger(Toast::info("Saved"))`);
- a modal `confirm` dialog;
- `anchored(Anchor::BottomLeft)` for HUD corners;
- `table` with a header row.

### 7. Restructure

`lib.rs` mixed the theme, components, the bundle API and the plugin. The plan is to split
it into `theme`, `components` and `bundles` modules, and `scenes` into submodules. The
crate root keeps re-exporting everything, so no downstream path changes. A `prelude`
gathers what a game needs.

### 8. Rewrite the challenges against the new API

The target is every challenge example at roughly 100 lines or fewer, with no `Color::`
outside a theme definition. That includes the challenges not yet started:

- 4, responsive menu (nine-slice buttons, breakpoint);
- 5, character editor (3D preview, scroll box, name input);
- 7, bug report form (dropdown, single-line and multi-line input).

### Later / not now

- **Theme files.** Load a theme from RON with hot reload. `Handle<Font>` and `Color`
  serde need the bevy `serialize` feature, which the trimmed library deliberately avoids.
- **Open-ended tokens.** `ColorToken` is a closed enum, so a game cannot add "mana". A
  `Custom(&'static str)` variant backed by a map in the theme would fix it, at the cost of
  typo safety.
- **Challenge 10 (design spec).** There is still no open spec to follow.
- **Deleting the old APIs.** Only after downstream has migrated.

## Status log

### 2026-11-19: plan items 1–8 implemented (uncommitted, for review)

| plan item | where | status |
|---|---|---|
| 1. Roles, variants, states, presets | `theme.rs`, `tokens.rs` | done. `surface` / `controls` sub-themes, a `Background` role, `Variant` + `VariantText`, `Selected` / `Checked` / `InteractionDisabled` handled in `apply_theme_tokens`, `dark()` / `light()` / `from_accent()` / `with_font()` |
| 2. Bindings | `bind.rs` | done. Text, meter and visibility from a resource; text and meter from another entity's component; two-way slider / checked / choice; write-back text input |
| 3. Screens | `screens.rs` | done. `add_screen`, `go_to`, `Back`, `screen_back`, `NavigateBack` |
| 4. Controls and focus | `controls.rs`, `focus.rs` | done. Slider, checkbox, toggle, choice, cycle, select, text input, text area; arrow / D-pad / stick navigation, gamepad A/B, focus ring, scroll-into-view, `UiNavigation` opt-out |
| 5. Images, nine-slice, responsive | `media.rs`, `responsive.rs` | done. `image`, `icon`, `nine_slice`, `skinned_button` (the palette tints the skin); `Responsive` / `stack_below` / `full_width_below` |
| 6. Small widgets | `overlay.rs`, `drag.rs`, `scenes/layout.rs` | done. Tooltip, toasts, `confirm`, drag and drop, `anchored`, `table`, `card`, `screen`, `backdrop` |
| 7. Restructure | `lib.rs` → `theme` / `components` / `bundles`; `scenes/` directory; `prelude` | done. Downstream paths are unchanged |
| 8. Challenges | `examples/` | done for 1–9 (three new: `4_responsive_menu`, `5_character_editor`, `7_bug_report`), 72–158 lines each, no hardcoded UI colours. Status table in `challenges.md` |

Also:

- `tests/widgets.rs` has 20 headless tests for the new layer.
- The README example is compiled as a doctest.
- `assets/` holds generated example images; `tools/make_assets.py` regenerates them.

Decisions worth a second look:

- **`scenes::button` is now `min_width` instead of a fixed `width`.** It fits its caption
  and stretches in a `card`. That is a visible change for existing scene-API users (listed
  in `migration-guide.md` §1.5).
- **Navigation only moves existing focus by default** (`UiNavigation::start_from_nothing
  = false`). Otherwise a game steering with the stick or arrow keys would wake the HUD's
  buttons. Menus start focus with `AutoFocus` on their first widget.
- **`choice` puts focus on each option, not on the radio group.** Bevy's radio group
  consumes all four arrows when focused, which traps vertical menu navigation.
- **Dropdown (`select`) is lava's own**, not feathers' (that one is editor-styled and about
  300 lines of popup / list machinery). It is a button plus an absolutely positioned list
  of `list_item`s. It closes on pick, on Escape / B and on a click outside.
- **`bind_input` is write-back only.** Seeding a text field from a resource needs
  `EditableText` editing API work that was not worth it yet.

Not done / next:

- **Checked by screenshot, interaction not tried.** Each challenge example was run once
  with Bevy's CI screenshot tool and the image was inspected; layout, theming, images,
  nine-slice skins and bindings render as intended. That pass fixed:
  - the slider and cycle stretch in cards;
  - `choice` wraps;
  - the dropdown caret was a missing glyph;
  - inventory counts were too large.
  Nothing has been *driven* yet: keyboard / gamepad focus movement, the dropdown popup
  over content below it, drag feel, `Responsive` at the breakpoint, typing.
- **The collapsible uses ▶ / ▼, which Bevy's default font lacks**, so it probably shows
  missing-glyph boxes. That predates this work and is unchanged; the dropdown now uses "v".
- Theme files (RON + hot reload), open-ended tokens, challenge 10: see "Later" above.
- Tabs (`bevy::ui_widgets` has them) and a number input are the obvious next controls.
- `bind_input` seeding, plus a `bind_selected` for list rows.
