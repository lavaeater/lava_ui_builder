# Lava UI Builder

Code-based UI for Bevy 0.20, with reasonable defaults and a theme you can swap at runtime.

It should be EASY to create a simple menu to start the game, make a choice in a game or
display some game info.

## Usage

Widgets are **scenes**, built with Bevy's `bsn!` macro. They take no theme argument:
colours and fonts resolve from the `LavaTheme` resource through tokens, so swapping the
theme repaints live UI. Behaviour (dragging, editing, checked state) is Bevy's own
headless widgets; lava adds the look and the wiring.

```rust,no_run
use bevy::prelude::*;
use lava_ui_builder::prelude::*;

#[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Menu { #[default] Main, Audio }

#[derive(Resource)]
struct Audio { volume: f32 }

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, LavaUiPlugin))
        .insert_resource(LavaTheme::from_accent(Color::srgb(0.9, 0.4, 0.15)))
        .insert_resource(Audio { volume: 0.7 })
        .bind_resource::<Audio>()               // lets widgets read and write `Audio`
        .init_state::<Menu>()
        .add_screen(Menu::Main, main_menu)      // spawned on enter, despawned on exit
        .add_screen(Menu::Audio, audio_menu)
        .screen_back(Menu::Audio, Menu::Main)   // Escape / gamepad B / `Back` buttons
        .add_systems(Startup, |mut c: Commands| { c.spawn(Camera2d); })
        .run();
}

fn main_menu() -> impl Scene {
    bsn! {
        @scenes::screen()
        Children [
            @scenes::card()
            Children [
                @scenes::title("My Game")
                --
                @scenes::button("Play") AutoFocus on(|_: On<Activate>| info!("play!"))
                --
                @scenes::button("Audio") Variant::Secondary @scenes::go_to(Menu::Audio)
                --
                @scenes::button("Quit") Variant::Danger
                on(|_: On<Activate>, mut exit: MessageWriter<AppExit>| { exit.write(AppExit::Success); })
            ]
        ]
    }
}

fn audio_menu() -> impl Scene {
    bsn! {
        @scenes::screen()
        Children [
            @scenes::card()
            Children [
                @scenes::header("Audio")
                --
                @scenes::slider(0.0, 1.0, 0.7) AutoFocus
                @scenes::bind_slider(|a: &Audio| a.volume, |a, v| a.volume = v)
                --
                @scenes::label("") @scenes::bind_text(|a: &Audio| format!("{:.0}%", a.volume * 100.0))
                --
                @scenes::button("Back") Variant::Ghost Back
            ]
        ]
    }
}
```

That menu works with the mouse, the keyboard (Tab, arrows, Enter, Escape) and a
controller (D-pad / stick, A, B), with a focus ring on the focused widget.

### What is in the box

| | |
|---|---|
| **Theme** | `LavaTheme` with roles (`surface`, `controls`, button variants, meters, fonts) and presets `dark()`, `light()`, `from_accent(color)`. |
| **Variants & states** | `Variant::{Primary, Secondary, Danger, Success, Ghost}` on any button; Bevy's `Selected` / `Checked` / `InteractionDisabled` markers pick the selected / disabled look. |
| **Layout** | `screen`, `ui_root`, `backdrop`, `card`, `row`, `column`, `grid`, `table`, `anchored(ScreenAnchor::BottomLeft, ..)`, `scrollable_list`, `Responsive` / `stack_below` / `full_width_below`. |
| **Text** | `title`, `header`, `label`, `caption`; recolour by patching a token (`ThemedTextColor(ColorToken::Danger)`). |
| **Controls** | `button`, `slider`, `checkbox`, `toggle`, `choice`, `cycle`, `select`, `text_input`, `text_area`, `meter`, `collapsible`, `list_item`. Each emits Bevy's `ValueChange<T>`. |
| **Bindings** | `bind_text`, `bind_meter`, `bind_visible` (resource → widget); `bind_text_to`, `bind_meter_to` (another entity's component → widget); `bind_slider`, `bind_checked`, `bind_choice`, `bind_input` (two-way). |
| **Screens** | `add_screen`, `go_to`, `Back`, `screen_back`, `NavigateBack`. |
| **Overlays** | `tooltip`, `Toast::success(..)`, `confirm(..)` dialogs. |
| **Media** | `image`, `icon`, `nine_slice`, `skinned_button` (a nine-slice skin tinted by the theme). |
| **Other** | `draggable` / `drop_target` / `Dropped`, `WorldFollower` (UI over a 3D entity), crawls. |

Things to know:

* **Themed widgets cannot be recoloured by patching a colour.** `apply_theme_tokens`
  rewrites `TextColor` and palettes every frame for entities carrying tokens. Patch the
  *token* or the *variant* instead; `scenes::text` / `scenes::button_colored` are the
  escape hatches for colour that is content, not style.
* **Sizes are layout, not theme**: patch `Node` (`@scenes::button("Ok") Node { width: px(200) }`).
* **`on(..)` needs a `Clone` observer system**, so capture `Copy` data (or clone into the
  closure) rather than moving a `String` in.
* **Bindings hold plain `fn`s** (closures that capture nothing), and each source type is
  registered once: `app.bind_resource::<R>()` / `app.bind_component::<C>()`.

### Examples

The ten "UI challenges" from [bevyengine/bevy#11100](https://github.com/bevyengine/bevy/discussions/11100)
are implemented as examples; see `challenges.md`. `cargo run --example game_menu` is a
good start. `examples/basic_layout.rs` is deliberately left on the older bundle-function
and `UIBuilder` APIs, which are frozen but still supported; see `bsn-migration.md` for
how the three APIs relate, and `roadmap.md` for where the library is going.

**Updating a project that depends on this crate?** `migration-guide.md` is the
step-by-step version: what actually breaks (short list), what improves for free, and how
to move a screen at a time rather than all at once.

## Issues & Simplification TODOs

### Bugs / Correctness

1. ~~add_themed_button_observe has hardcoded "PLAAAY" as button text (builder.rs:762)~~
   ~~The function accepts a closure f but creates the text entity with Text::new("PLAAAY") instead of taking a text parameter like add_button_observe does.~~
   ~~The text can only be fixed via the ButtonBuilder closure workaround. Add a text parameter and remove the hardcoded string.~~ **DONE**

2. ~~WidgetsButton inserted twice in add_themed_button_observe (builder.rs:772, 778)~~
   ~~Two separate .insert() calls on the same entity both include WidgetsButton. The second block (lines 776–786) duplicates components from the first block (lines 769–774). Merge into one insert.~~ **DONE**

3. ~~feathers_button_with_overrides creates a ButtonBuilder with text_entity: None (button_builder.rs:162)~~
   ~~Calling .text(), .font_size(), or .text_color() on this builder silently does nothing. The feathers button() spawns text as a child, so there's no handle to it — but this should either be documented clearly or the text entity should be found after spawning.~~ **DONE** — added doc comment warning that `.text()`, `.font_size()`, `.text_color()` are no-ops on this builder.

4. ~~build() returns wrong entity when parent_stack is non-empty (builder.rs:123)~~
   ~~Returns parent_stack[0] (the oldest ancestor) rather than the actual root that was spawned in new(). Store the root entity explicitly at construction time instead.~~ **DONE**

  ---                                                                                               
### Duplication

5. ~~Three nearly-identical button setup methods: add_themed_button, add_button_observe, add_themed_button_observe~~
   ~~All three: spawn a child, build a Node + colors + InteractionPalette, spawn a text child, create a ButtonBuilder, run a panic-safe closure, restore stack. Extract a private spawn_button_inner(text, node, extras, handler, f) helper that all three delegate to.~~ **DONE** — extracted `spawn_button_inner` private helper.

6. ~~spawn_collapsible_section in systems.rs duplicates the collapsible logic from with_collapsible in builder.rs~~
   ~~Two separate implementations of the same widget. Remove spawn_collapsible_section (or make with_collapsible call a shared internal function it also exposes).~~ **DONE**

7. ~~flex_dir_row / flex_dir_column / flex_direction_row are aliases for flex_row / flex_column (builder.rs:384–392)~~
   ~~Three methods, one behaviour. Remove the aliases and keep only flex_row / flex_column.~~ **DONE**

8. ~~add_collapsible / add_collapsible_collapsed just call with_collapsible with a boolean (builder.rs:856–866)~~
   ~~Not worth two separate methods — the boolean makes intent clear enough. Remove them or at least keep only with_collapsible.~~ **DONE**

  ---                                                                                                                                                 
### Hardcoded values that should live in the theme

9. ~~Collapsible toggle button colors are hardcoded in three places~~
   ~~Add a collapsible_bg, collapsible_bg_hovered, collapsible_bg_pressed to ButtonTheme (or a dedicated CollapsibleTheme) and read from the resource in the system.~~ **DONE**

10. ~~add_themed_button_observe hardcodes width: 150px, height: 75px (builder.rs:748–749)~~
    ~~These should come from theme.button.width / theme.button.height, just like add_button_observe uses button_theme.border_width.~~ **DONE**

11. ~~adapt_ui_scale hardcodes base width 1920.0 (lib.rs:361)~~
    ~~Move this to LavaTheme.ui_width (the field already exists) so users can configure it.~~ **DONE**

  ---                                                                                                                                                 
### Minor

12. ~~with_text has six Option parameters — use a builder or a struct instead~~
    ~~with_text(text, None, None, None, None, None) is unreadable. Introduce a TextStyle struct with defaults, or at minimum provide named convenience constructors.~~ **DONE** — introduced `TextStyle` struct with `size()`, `color()`, `size_color()` convenience constructors. Updated all callers.

13. VecDeque import in builder.rs is redundant — VecDeque is only used in lib.rs
    **N/A** — `VecDeque::new()` is used in builder.rs (lines 31 and 49). Import is required.
