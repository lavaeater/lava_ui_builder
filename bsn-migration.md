# BSN migration plan for `lava_ui_builder`

Status: **Phases 0-5 are implemented.** The scene API is the preferred way to build UI
in this crate, every example except `basic_layout` is on it, and `basic_layout` stays on
the old APIs on purpose as the reference for unmigrated code. See §8 for the log and for
what the implementation corrected in this document. Written against **bevy 0.19.1**, verified
against the vendored sources in `~/.cargo/registry/src/*/bevy_scene-0.19.1/` and
`bevy_feathers-0.19.1/`, plus bevy's own `examples/scene/bsn.rs` and
`examples/ui/widgets/feathers_counter.rs`. Every BSN construct proposed below was
compiled against this crate first — see Appendix A.

---

## 1. TL;DR

BSN replaces roughly everything the **bundle-function API** does, and most of what the
**imperative `UIBuilder`** does, with less code and better composition:

* `children![]` + `Children::spawn(SpawnWith(..))` → `Children [ ... ]`
* the ~80 one-property `Node` setters on `UIBuilder` → one `Node { .. }` patch, merged
  field-by-field across composed scenes
* `.observe(handler)` plumbed through three builder layers → `on(|ev: On<Activate>| ..)`
  written inline next to the node it belongs to
* theme overrides threaded as `Option<TextStyle>` / `ButtonBuilder` closures → patching:
  `lava_button("Play") Node { width: px(200) }`

What BSN does **not** give us in 0.19: no `.bsn` asset files, no hot reload, no scene
caching for function-scenes. It is a code-only workflow this release.

Recommendation: **add** a scene API next to the existing ones (Phase 1–2), migrate the
examples (Phase 4), then freeze and deprecate `UIBuilder` rather than porting its 120-odd
methods. Do the bug fixes in §3 *first* — otherwise they get copied into the new API.

---

## 2. What BSN actually is in 0.19

Everything below is in `bevy::prelude` once the `bevy_scene` feature is on (it is —
see §5.0): `bsn!`, `bsn_list!`, `on`, `template_value`, `Scene`, `SceneList`,
`SceneComponent`, `SpawnSystem`, `WorldSceneExt`, `CommandsSceneExt`,
`EntityCommandsSceneExt`, `PatchTemplate`.

### The macro

```rust
fn button(label: &str) -> impl Scene {
    bsn! {
        Button
        Node { width: px(150), height: px(65), border_radius: BorderRadius::MAX }
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15))
        Children [(
            Text(label)
            TextColor(Color::srgb(0.9, 0.9, 0.9))
        )]
    }
}
```

Facts that matter for us:

| Concept | Detail |
| --- | --- |
| **Patching** | `Node { width: px(10) }` writes *only* `width`. Later patches to the same component merge; unset fields fall back to the previous patch or `Default`. This is the whole reason our `modify_node`/`TextStyle::Option` machinery can go away. |
| **Composition** | A `Scene`-returning function call inside `bsn!` splices its entries in place: `lava_button("Play") Node { width: px(200) }` overrides just the width. Tuples of scenes are scenes. |
| **Children** | `Children [a, b]` = two entities; `Children [a b]` = one entity with both components. Any `RelationshipTarget` works, not just `Children`. |
| **Dynamic content** | `{expr}` interpolates a Rust expression. `Vec<S: Scene>` implements `SceneList`, so `Children [ {rows} ]` splices a runtime-built list. |
| **Conditionals** | `Option<S>` implements `Scene`, and `Option<L>` implements `SceneList` **only when `L` is itself a `SceneList`**. In a `Children [ .. ]` position an interpolated value must be a *list*, so an optional child is `cond.then(\|\| bsn_list![..])`, not `cond.then(\|\| bsn!{..})`. |
| **Observers** | `on(|ev: On<Activate>, ..| { .. })` attaches an entity observer. **The system must be `Clone`** (`OnTemplate` is `impl<I: IntoObserverSystem<..> + Clone>`, `bevy_scene-0.19.1/src/scene.rs:519`). Our current signatures take bare `impl IntoObserverSystem` — see §6. |
| **Named entities** | `#Root` sets `Name("Root")` *and* registers the entity for reference within the same `bsn!` scope. `Component { field: #Root }` resolves to an `Entity` at spawn time, upward or downward in the tree. |
| **Component requirements** | To appear in `bsn!` a component needs either `Default + Clone`, or `#[derive(FromTemplate)]`. `FromTemplate` is required for fields needing spawn context — `Handle<T>` (asset paths) and `Entity` (from `#Name`). Deriving both `FromTemplate` and `Default` is an error. |
| **Asset fields are templates** | A `Handle<T>` field is patched as `HandleTemplate<T>`, whose `From<impl Into<AssetPath>>` is why bevy's example writes a *path* string. To patch one with an already-resolved handle, pass `HandleTemplate::Handle(h)`. `TextFont.font` adds a layer: it is a `FontSource`, so the patch is `FontSourceTemplate::Handle(HandleTemplate::Handle(h))` — see `scenes::font_source`. |
| **Enums** | Need per-variant defaults; use the `VariantDefaults` pseudo-derive (implied by `FromTemplate`). |
| **Scene components** | `#[derive(SceneComponent)] #[scene(MyProps)] struct MyWidget;` + `fn scene(props) -> impl Scene`. Consumers write `@MyWidget { @prop: value }`. A debug-build hook *errors* if the component is ever inserted without its scene. |
| **Props vs. fields** | Inside `@MyWidget { .. }`, a plain `field: v` patches a field of the *component*; a `@name: v` entry is a **prop** passed to `MyWidget::scene`. Mixing them up is an `E0609 no field ... on type` error. |
| **Values are BSN, not Rust** | A field/prop value must be BSN value syntax (literal, path, struct/tuple literal, `#Name`). Any other Rust expression needs braces: `@label: {"Click".to_string()}`. |
| **Spawning** | `commands.spawn_scene(s)` / `world.spawn_scene(s)` are immediate and fail if asset deps are unloaded; `queue_spawn_scene` waits for deps and spawns in the `SpawnScene` schedule (between `Update` and `PostUpdate`). `scene_fn.spawn()` turns a scene fn into a startup system. |
| **Applying to an existing entity** | `EntityCommands::apply_scene` / `queue_apply_scene`, and `queue_spawn_related_scenes::<Children>(list)`. This is the bridge for runtime rebuilds and for `UIBuilder` interop. |
| **Caching** | The `:` prefix caches a resolved scene — but only for scene *assets*. Using it on a function scene or `SceneComponent` is a compile error in 0.19. |

### How bevy's own widget library uses it

`bevy_feathers` is the reference design, and it answers the theme question for us
(`bevy_feathers-0.19.1/src/controls/button.rs:57`):

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(FeathersButtonProps)]
pub struct FeathersButton;

pub struct FeathersButtonProps {
    pub caption: Box<dyn SceneList>,   // children as a prop
    pub variant: ButtonVariant,
    pub corners: RoundedCorners,
}

impl FeathersButton {
    fn scene(props: FeathersButtonProps) -> impl Scene {
        bsn! {
            Node { height: size::ROW_HEIGHT, border_radius: {props.corners.to_border_radius(4.0)} }
            Button
            template_value(props.variant)
            Hovered
            ThemeBackgroundColor(tokens::BUTTON_BG)     // <- token, not a Color
            InheritableThemeTextColor(tokens::BUTTON_TEXT)
            Children [ {props.caption} ]
        }
    }
}
```

Two things to steal: **props structs with `Default`** (a scene component's props must be
`Default`), and **theme tokens as components** rather than colors baked into the scene.

---

## 3. Code review of the current crate

Baseline: `cargo build` is clean. `src/` is 2 239 lines across four files; examples add
2 100. Two public APIs (bundle functions in `lib.rs`, `UIBuilder`/`ButtonBuilder` in
`builder.rs`/`button_builder.rs`) that do not share widget implementations.

### 3.1 Bugs — fix these before migrating

**B1. `InteractionPalette` never fires on any `UIBuilder` button.** (high)
`systems::apply_interaction_palette` (`src/systems.rs:191-201`) queries `&Interaction`.
`Interaction` is only added as a required component of the *legacy* `bevy_ui::widget::Button`
(`bevy_ui-0.19.1/src/widget/button.rs`: `#[require(Node, FocusPolicy::Block, Interaction)]`).
Every builder-side button instead inserts `bevy::ui_widgets::Button`, which requires only an
`AccessibilityNode` and tracks state via `Hovered`/`Pressed`:
`builder.rs:742` (`list_item`), `:771` (`icon_button`), `:802` (`delete_button`), `:923`
(`add_themed_button`), `:961` (`add_button_observe`), `:1005` (`add_themed_button_observe`),
plus `lib.rs:491` (`spawn_list_item`). Those entities never get an `Interaction`, so the
query never matches and hover/press colors are dead. The bundle-function
`themed_button_with_node` (`lib.rs:301`) uses the prelude `Button` (= legacy) and *does*
work — which is why this has probably gone unnoticed.

Fix: drive the palette off `Hovered` + `Pressed` (the headless-widget state) instead of
`Interaction`, and stop mixing the two `Button` types. A `Changed<Hovered>`/`Added<Pressed>`/
`Removed<Pressed>` system covers it. This is a prerequisite for the migration because the
new scenes should carry one, working palette component.

**B2. Two `Button` types, two behaviours.** (medium)
`lib.rs:301` uses the legacy button (gets `Interaction`, `FocusPolicy`, no `Activate`
event, no tab focus); `builder.rs` uses `ui_widgets::Button` (emits `Activate`, has
`TabIndex(0)`, `EntityCursor`). So keyboard activation and cursor feedback exist on one
half of the library and not the other. Pick `ui_widgets::Button` everywhere — it is the
one bevy is moving to, and it is what `Activate`-based observers in the examples expect.

**B3. `handle_collapse_toggle` also writes `BackgroundColor`.** (low)
`systems.rs:42-83` sets background colors from the theme on `Interaction` changes.
If a collapsible toggle ever also gets an `InteractionPalette`, the two systems fight in
an order-dependent way. After B1 this should be one mechanism: give the toggle a palette
and delete the color code from the toggle system.

### 3.2 Design issues BSN resolves

**D1. The `Node` setter wall.** `builder.rs` has ~80 methods (`width_px`, `height_px`,
`min_width`, `justify_start`, …) that each queue an `entry::<Node>().and_modify(closure)`
command — one boxed closure and one archetype lookup per property per entity. BSN's
patching gives the same "override one field" semantics as a plain struct literal,
resolved before the entity exists. This is the single biggest code deletion available.

**D2. `modify_node` silently no-ops without a `Node`.** `entry::<Node>().and_modify(..)`
does nothing if the component is absent. `child()` always spawns `Node::default()`, so
this is safe today — but `start_from_entity` (`builder.rs:36`) accepts any entity, and
every setter on a `Node`-less root vanishes without a warning. BSN patches insert the
component if missing.

**D3. Style overrides are `Option` soup.** `TextStyle`'s five `Option` fields, the
`Option<TextStyle>` parameter on `with_text`, and the whole `ButtonBuilder` (25 methods
whose only job is to patch the button node after the fact) exist to express "same widget,
one field different". That is exactly what patching is: `lava_label("hi") TextFont { font_size: px(11) }`.

**D4. `catch_unwind` in the builder.** `builder.rs:86`, `:109`, `:892` wrap user closures
in `catch_unwind(AssertUnwindSafe(..))` purely to restore `current_entity`/`parent_stack`
before resuming the unwind. A panic inside a Bevy system is already fatal to the app, so
this buys nothing but unwind tables and an `AssertUnwindSafe` assertion nobody checks.
Scenes are values — there is no traversal state to restore, so this disappears with the
builder.

**D5. `themed_button_with_node` spawns a redundant wrapper.** `lib.rs:272-317` returns an
outer entity with a bare `Node::default()` whose only child is the real button, spawned
through a `SpawnWith` closure so it can `.observe()`. The wrapper is an extra flex item
with default layout (it participates in the parent's flex and can squash the button), and
the button gets `Node::default()` inserted and then overwritten by `.insert(button_node)`.
In BSN the observer attaches inline, so the wrapper and the double insert both go away.

**D6. Themes are baked at build time.** `UIBuilder::new(commands, Some(theme.clone()))`
copies the theme into the builder and each widget reads colors out of it at spawn. Nothing
links a spawned node back to the theme, so `examples/dark_light_theme.rs` switches themes
by *destroying and rebuilding the entire UI* (`start_from_entity(.., clear_children: true)`).
Feathers solves this with token components + a system that repaints on
`resource_changed::<UiTheme>`. BSN doesn't force the change, but the migration is the
natural moment to make it (see Phase 2).

### 3.3 Documentation drift (cheap fixes)

* `CLAUDE.md` names `LavaUiBuilderPlugin`, `interaction_palette_system`,
  `collapsible_toggle_system`, `adapt_ui_scale` in `systems.rs`. The real names are
  `LavaUiPlugin` (`lib.rs:514`), `apply_interaction_palette`, `handle_collapse_toggle`,
  and `adapt_ui_scale` lives in `lib.rs`. It also misses `handle_scroll_input`,
  `sync_progress_bars` and `world_follower_system`.
* `CLAUDE.md`: "There are no tests in this crate" — `systems.rs:209` opens a `mod tests` with four.
* `Cargo.toml`: the comment says `"ui"` is expanded by hand into `ui_api` +
  `ui_bevy_render` to avoid `default_platform`'s `android-game-activity`, but the
  dependency line plainly uses `features = ["ui", "bevy_feathers"]`. Either the comment or
  the workaround is stale, and the downstream civilization build is the one that pays.
  Relevant here because `ui` is also what currently pulls in `scene` → `bevy_scene` (§5.0).
* `Cargo.toml` declared a `feathers` feature, but nothing in `src/` was gated on
  `feature = "feathers"` — `bevy_feathers` is an unconditional dependency and
  `button_builder.rs`'s feathers methods always compile. **Resolved:** the feature was
  removed rather than wired up. `--features feathers` is now an error instead of a no-op,
  which is the point: it never did anything.

---

## 4. API mapping

| Today | BSN |
| --- | --- |
| `ui_root("Menu")` (`impl Bundle`) | `fn ui_root() -> impl Scene` |
| `children![(a), (b)]` | `Children [ (a), (b) ]` |
| `Children::spawn(SpawnWith(\|p\| ..))` (used only to get `.observe`) | `on(..)` inline — wrapper entity deleted |
| `header(t, &theme.text)` / `label(..)` | `lava_header(t)` / `lava_label(t)` scene fns |
| `TextStyle::size_color(11.0, c)` | `TextFont { font_size: px(11) } TextColor(c)` patches |
| `themed_button(text, action, &theme.button)` | `@LavaButton { label: .. } on(action)` |
| `themed_button_with_node(.., node)` | `@LavaButton { .. } Node { width: px(380) }` |
| `ButtonBuilder::{width,height,bg_color,..}` | patches after the widget call |
| `ui.child()` / `.parent()` / `.with_child(..)` | nesting in `Children [ .. ]` |
| `ui.foreach_child(items, ..)` | `Children [ {items.map(..).collect::<Vec<_>>()} ]` |
| `if cond { ui.with_child(..) }` | `Children [ {cond.then(\|\| bsn!{ .. })} ]` |
| `progress_bar(v, w, h, fill, bg)` | `@LavaProgressBar { value: v }` |
| `CollapseToggleButton { target: e }` wired by hand | `#Section` + `CollapseToggleButton { target: #Section }` |
| `spawn_list_item(parent, ..)` (raw `ChildSpawner` escape hatch) | just a scene fn — no escape hatch needed |
| `start_from_entity(.., clear_children: true)` rebuild | `despawn_related::<Children>()` + `queue_spawn_related_scenes::<Children>(..)` |
| `UIBuilder` patching an existing entity | `commands.entity(e).apply_scene(bsn!{ .. })` |

---

## 5. Migration phases

### Phase 0 — prerequisites

1. **Fix B1 and B2.** New scenes must carry a palette component that actually works.
2. **Feature check.** `bevy_scene` is enabled today only transitively: our `ui` feature
   → `scene` → `bevy_scene` (`bevy-0.19.1/Cargo.toml`). If the hand-expansion described in
   the `Cargo.toml` comment (`ui_api` + `ui_bevy_render`) is ever restored, `bsn!` vanishes
   from the prelude. Add `"bevy_scene"` to the feature list explicitly, now.
3. **Plugin.** `ScenePlugin` is in `DefaultPlugins`; document that consumers building a
   custom plugin set must add it, or have `LavaUiPlugin` add it defensively.
4. **Make our components BSN-usable:**

   | Component | Today | Needs |
   | --- | --- | --- |
   | `InteractionPalette` | `Clone` | `+ Default` |
   | `Collapsible` | — | `+ Default + Clone` |
   | `CollapsibleContent { parent: Entity }` | — | `#[derive(FromTemplate)]` |
   | `CollapseToggleButton { target: Entity }` | — | `#[derive(FromTemplate)]` |
   | `ProgressBar` | `Clone` | `+ Default` |
   | `ProgressBarFill` | — | `+ Default + Clone` |
   | `WorldFollower { target: Entity, .. }` | `Clone` | `#[derive(FromTemplate)]` (drop `Clone`-only) |
   | `ButtonTheme` / `TextTheme` | `Clone + Default` | fine as props fields |

   Remember: `FromTemplate` and `Default` cannot both be derived on the same type; the
   derive generates `XTemplate::default()` for that purpose.

### Phase 1 — a `scenes` module beside the existing APIs

New `src/scenes.rs`, re-exported from `lib.rs`. Nothing is removed yet; the bundle
functions and `UIBuilder` keep working.

```rust
pub fn ui_root() -> impl Scene {
    bsn! {
        Node {
            position_type: PositionType::Absolute,
            width: percent(100), height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_direction: FlexDirection::Column,
            row_gap: px(20),
        }
        template_value(Pickable::IGNORE)
    }
}

pub fn lava_label(text: impl Into<String>, theme: &TextTheme) -> impl Scene {
    let (font, size, color) = (theme.font.clone(), theme.label_size, theme.label_color);
    let text = text.into();
    bsn! {
        Text({text})
        TextFont { font: {font}, font_size: {px(size)} }
        TextColor({color})
    }
}
```

Note the shape: **pull owned values out of the theme before the macro**. `Scene` is
`Send + Sync + 'static`, so a scene may not borrow the theme.

Callers compose and patch:

```rust
commands.spawn_scene(bsn! {
    ui_root()
    Children [
        lava_header("Main Menu", &theme.text),
        lava_label("subtitle", &theme.text) TextColor(Color::WHITE),   // patch
    ]
});
```

### Phase 2 — widgets as scene components, theme as tokens

Promote the widgets consumers actually compose with (`button`, `progress_bar`,
`collapsible`, `list_item`) to `SceneComponent`s so downstream code writes `@LavaButton`:

```rust
#[derive(SceneComponent, Default, Clone)]
#[scene(LavaButtonProps)]
pub struct LavaButton;

#[derive(Default)]
pub struct LavaButtonProps {
    pub caption: Box<dyn SceneList>,   // or String for the simple case
    pub variant: ButtonVariant,
}

impl LavaButton {
    fn scene(props: LavaButtonProps) -> impl Scene {
        bsn! {
            Node { justify_content: JustifyContent::Center, align_items: AlignItems::Center }
            bevy::ui_widgets::Button
            Hovered
            TabIndex(0)
            EntityCursor::System(SystemCursorIcon::Pointer)
            LavaBackground(tokens::BUTTON_BG)       // token, resolved by a system
            Children [ {props.caption} ]
        }
    }
}
```

Callers then write `@LavaButton { @caption: {bsn_list![Text("Play")]} }` — note the `@`
on prop names.

The token half is the answer to **D6**: introduce `LavaBackground(ThemeToken)` /
`LavaTextColor(ThemeToken)` components plus a system that repaints them on
`resource_changed::<LavaTheme>`, mirroring `bevy_feathers::theme`. Live theme switching
then costs one resource write instead of a full UI rebuild, and
`examples/dark_light_theme.rs` loses its `start_from_entity` rebuild entirely.

If that is too big a step, an intermediate version keeps `Color`s in the props and simply
accepts that theme switches still rebuild. The scene API does not depend on the token
decision — but making it later is a breaking change to every widget's props, so decide
before Phase 4.

### Phase 3 — dynamic and runtime UI (the `UIBuilder` replacement)

The three things the builder is genuinely used for, and their BSN forms:

```rust
// data-driven lists (replaces foreach_child)
let rows: Vec<_> = items.iter().map(|it| {
    let name = it.name.clone();
    bsn! { lava_list_item() Children [ Text({name}) ] }
}).collect();
bsn! { Node { flex_direction: FlexDirection::Column } Children [ {rows} ] }

// conditional sections
Children [ {show_footer.then(|| bsn! { lava_label("footer", &theme.text) })} ]

// runtime rebuild (replaces start_from_entity(.., clear_children: true))
commands.entity(panel)
    .despawn_related::<Children>()
    .queue_spawn_related_scenes::<Children>(bsn_list![ /* fresh rows */ ]);

// patch an existing entity without rebuilding it
commands.entity(panel).apply_scene(bsn! { Node { display: Display::None } });
```

`apply_scene` is also the **interop bridge**: add
`UIBuilder::apply_scene(&mut self, scene: impl Scene)` so half-migrated code can drop a
scene into a builder tree at the current entity. That lets the examples migrate one
subtree at a time.

After the examples are ported, freeze `UIBuilder` (no new methods) and mark it
`#[deprecated(note = "use the scene API")]`. Porting its 120 methods is not worth it: the
Node setters are subsumed by patching, and the tree-walking methods have no analogue.

### Phase 4 — examples

Port in this order, smallest and most representative first, keeping each example runnable:

1. `basic_layout` — pure static tree; the "hello world" of the new API.
2. `game_menu` — buttons + observers; proves `on()` and the `Clone` constraint (§6).
3. `scoreboard` / `hud` — dynamic text driven by systems; proves marker components still
   work for `Query<&mut Text, With<Marker>>` updates.
4. `inventory` — data-driven grid; proves `Vec<impl Scene>` interpolation.
5. `dark_light_theme` — the token system from Phase 2; the payoff example.
6. `complex_example` (475 lines) — last, as the integration test.

`CLAUDE.md` says validation is by running examples, so each port is its own verification
step: `cargo run --example <name>`.

### Phase 5 — positioning for `.bsn` assets

0.19 ships no asset loader, but the shape of our code decides how painful that release is:

* keep widgets as **free functions / scene components with props structs**, not closures —
  that is what a future asset can reference;
* prefer `FontSourceTemplate::Handle("fonts/..")`-style asset *paths* in scenes over
  pre-resolved `Handle<Font>` where possible (then spawn with `queue_spawn_scene`, which
  waits for the load);
* avoid putting behaviour that can't be expressed declaratively inside widget scenes.

---

## 6. Gotchas found while reading the source

1. **`on()` requires `Clone`.** `OnTemplate: impl<I: IntoObserverSystem<E,B,M> + Clone>`
   (`scene.rs:519`). Our public signatures (`themed_button`, `add_button_observe`,
   `list_item`, …) take bare `impl IntoObserverSystem`, and a closure capturing a
   non-`Clone` value will not compile in a scene. Widget signatures must add `+ Clone`,
   and the migration guide for downstream code has to say so.
2. **Scenes can't borrow.** `Scene: Send + Sync + 'static`. Clone/copy theme values out
   before the `bsn!` block; `&TextTheme` parameters are fine, stored references are not.
3. **Never insert a `SceneComponent` directly.** A debug-build `on_add` hook logs an error
   ("spawned with the scene component X, but without its scene"). `commands.spawn(LavaButton)`
   is now a bug — only `@LavaButton` / `spawn_scene` are valid.
4. **`EntityTemplate` must not be stored.** `#Name` resolves to an `Entity` only during
   spawn. Components hold `Entity` and derive `FromTemplate`; they never hold the template.
5. **Name scope is per `bsn!` invocation.** A `#Section` in a widget's scene is invisible to
   the caller — which is exactly why `CollapseToggleButton { target: #Section }` has to be
   written in the *same* macro invocation that declares `#Section`, i.e. in the collapsible
   widget's own scene, not spread across two functions.
6. **Spawn timing differs.** `spawn_scene` is immediate; `queue_spawn_scene` lands in the
   `SpawnScene` schedule, between `Update` and `PostUpdate`. Code that spawns and then
   queries the result in the same system must use the immediate form (and handle its
   `Result`).
7. **Caching (`:`) is asset-only.** `: lava_button` is a compile error in 0.19.
8. **Enums need per-variant defaults** (`VariantDefaults`) — relevant if `ButtonVariant`-like
   enums of ours ever appear as patch targets.
9. **`Vec<Box<dyn SceneList>>` exists** for heterogeneous lists, and `Box<dyn SceneList>` is
   the idiomatic props type for "children passed in by the caller" (feathers' `caption`).

---

## 7. Decisions taken

* **Tokens, not colors in props.** Decided in favour of tokens (§8, Phase 2). Font tokens
  came along because colors alone still leave every text widget needing a theme argument.
* **The bundle-function API and `UIBuilder` stay, undeprecated.** They are superseded, and
  said so in the docs, but no `#[deprecated]` attribute was added: `civilization` consumes
  this crate as a path dependency and lints at pedantic level, so the attribute would turn
  a documentation change into a wall of warnings in an unrelated build. It is a one-line
  change whenever that build is ready for it.
* **Sizes are not tokens.** Layout is patched (`scenes::button("Play") Node { width: px(220) }`).
  A consequence is that `LavaTheme.button.width`/`height` are unused by the scene API.
* **List-item / icon / delete colors are consts, not tokens.** Promoting them would mean
  new `LavaTheme` fields, which breaks any downstream construction that does not end in
  `..Default::default()`.

### Still open

* **How much does `civilization` use `UIBuilder`?** That governs whether the builder ever
  gets removed, or just stops growing.

## 8. Implementation log

### Landed

**Phase 0 — prerequisites**

* **B1 fixed.** `systems::apply_interaction_palette` now reads `Hovered` + `Pressed`
  instead of `Interaction`, so the palette works on `ui_widgets::Button` entities. The
  query is deliberately unfiltered: `Pressed` is *removed* on release and removal is
  invisible to `Changed`/`Added`, so the color is recomputed each frame and written only
  when it differs, which keeps `BackgroundColor` change detection honest.
* **B2 fixed.** Every button path is now `bevy::ui_widgets::Button` + `Hovered`:
  `themed_button_with_node` (`lib.rs`), `add_button` and `add_themed_button`
  (`builder.rs`, the latter had no `Hovered` at all), and the collapsible toggle.
* **B3 fixed.** `handle_collapse_toggle` is gone. Toggling is now
  `systems::toggle_collapsible_on_activate`, a **global observer** on `Activate`
  registered by `LavaUiPlugin`, so it works for buttons spawned by any of the three APIs
  and responds to ENTER/SPACE. The toggle's colors come from an `InteractionPalette`
  like every other button, instead of being written by the toggle system.
* **Component derives** added as planned: `Default` on `InteractionPalette`,
  `ProgressBar`, `ProgressBarFill`, `Collapsible` (+`Clone`), and `FromTemplate` on the
  three `Entity`-holding components (`CollapseToggleButton`, `CollapsibleContent`,
  `WorldFollower`).
* `"bevy_scene"` is now an explicit feature in `Cargo.toml` rather than a transitive
  edge of `"ui"`.

**Phase 1 — `src/scenes.rs`**

`ui_root`, `header`, `label`, `button`, `progress_bar`, `collapsible`, plus the
`font_source` bridge. Nothing was removed: the bundle functions and `UIBuilder` are
untouched and still compile.

**Verification (final state)**

* `tests/scenes.rs` — 13 headless tests (`MinimalPlugins` + `AssetPlugin` + `ScenePlugin`)
  asserting the *shape* of each spawned scene, including that `collapsible`'s `#Section`
  reference resolves to the real section entity on both children, and that a caller's
  `TextColor` patch overrides the color without disturbing the text.
  (`MinimalPlugins` + `AssetPlugin` + `ScenePlugin`) covering scene shape, `#Name`
  resolution, token resolution, live re-theming, patching, `replace_children`, and the
  legacy-`Interaction` fallback. Plus the 4 existing `follower_axis` unit tests.
* All eight scene examples built and smoke-run: window up, no panics, no
  scene-resolution errors.
* `cargo clippy --all-targets --all-features` clean under the pedantic lint set.

### What the implementation corrected in this plan

-1. **Braces are for component values, not function arguments.** `scenes::text(format!(..), ..)`
   needs no braces; `Text({text})` does. The rule is that a scene-function call is ordinary
   Rust, while a component's field or tuple value is BSN syntax.
0. **Associated consts work bare too**, e.g. `Pickable::IGNORE` as a component entry.
0. **Bare enum variants work as BSN values.** `ThemedPalette { none: ColorToken::ButtonBg }`
   needs no braces and no `VariantDefaults`; the special-casing in §2 applies to patching
   enum *fields*, not to passing a fieldless variant.
1. **Props use an `@` prefix.** `@LavaButton { @caption: .. }`. A bare `field: v` inside
   the braces patches the *component*, giving `E0609 no field ... on type &mut LavaButton`.
2. **Optional children are `Option<impl SceneList>`**, not `Option<impl Scene>` — see the
   table in §2.
3. **Theme fonts need the template bridge** described in §2; a `Handle<Font>` cannot be
   assigned to `TextFont.font` inside `bsn!`.
4. **Values must be BSN syntax.** `Text(label.into())` does not parse; `Text({label})`
   does. This bites most when converting existing `impl Into<String>` widget signatures.

**Phase 2 — theme tokens (`src/tokens.rs`)**

The open question in §7 was decided in favour of tokens. Widgets no longer take a theme
argument at all:

```rust
scenes::button("Play")                     // was: scenes::button("Play", &theme.button)
scenes::collapsible("Details", false, content)
```

* `ColorToken` / `FontToken` are **enums**, not feathers' string newtype. The token set is
  exactly the fields of `LavaTheme`, so an enum makes a typo a compile error and the
  resolver exhaustive. The trade-off is that downstream crates cannot invent tokens; if
  that is ever wanted this becomes a newtype plus a map, as feathers does it.
* Components: `ThemedBackground`, `ThemedTextColor`, `ThemedBorderColor`, `ThemedFont`,
  and `ThemedPalette` (which fills an `InteractionPalette` from three tokens). Each uses
  `#[require(..)]` so the painted component comes along automatically.
* `apply_theme_tokens` resolves all five, ordered `.chain()`-ed *before*
  `apply_interaction_palette`: tokens produce the palette, the palette produces the
  `BackgroundColor` for the current hover/press state.
* **Font tokens were not optional.** Colors alone would have left every text widget still
  taking a theme argument just to learn its font size.
* **Sizes deliberately stayed out.** Button width/height/border are consts in `scenes.rs`,
  patched via `Node` at the call site. Layout is what patching is *for*, and unlike a
  color it does not need to follow a runtime theme change.

One consequence worth knowing: patching a bare `TextColor` over a themed widget does not
stick, because the token system repaints it next frame. Override the *token* instead
(`scenes::label("x") ThemedTextColor(ColorToken::HeaderText)`), which is also what keeps
the override following theme switches.

**Phase 3 — the rest of the widgets, and the bridge**

`text`, `section_label`, `row`/`column`/`panel`/`grid`/`centered`/`spacer`, `side_panel`,
`scrollable_list(_bounded)`, `list_item`, `icon_button`, `delete_button`, and
`button_colored`.

Two kinds of widget emerged, and the split is worth keeping in mind when adding more:

| Themed | Explicit |
| --- | --- |
| `label`, `header`, `button`, `collapsible` | `text`, `button_colored`, `list_item`, `icon_button` |
| carries tokens, follows theme swaps | carries concrete colors |
| **cannot** be recolored by patching | patch freely |

The reason for the first row's caveat is mechanical: `apply_theme_tokens` rewrites
`TextColor` / `InteractionPalette` every frame, so a patched color survives exactly one
frame. `button` and `button_colored` are therefore built from a shared `button_base` and
differ by exactly one entry.

Interop, so a tree can move over a subtree at a time:

* `UIBuilder::apply_scene` / `scene_child` / `scene_children`
* `scenes::replace_children(commands, parent, list)` -- the scene answer to
  `start_from_entity(.., clear_children: true)`. The parent entity survives, so stored
  `Entity` ids, marker queries and `ScrollPosition` all keep working.

**Phase 4 — examples**

All ported: `bsn_layout` (new, the scene port of `basic_layout`), `buttons`, `scoreboard`,
`dark_light_theme`, `game_menu`, `hud`, `inventory`, `complex_example`. `basic_layout`
stays on the old APIs as the unmigrated reference.

Three of them were carrying copies of library systems, and two of those copies were the
`Interaction` bug from §3.1 in example form: `buttons` had its own palette system, and
`complex_example` had its own scroll, collapse-toggle and collapsible-visibility systems.
Both now just add `LavaUiPlugin`.

`game_menu` needed a real behavioural change: its selection highlight wrote
`BackgroundColor` directly, which the palette system now owns and would overwrite the
next frame. It swaps the whole `InteractionPalette` instead. **Any downstream code that
writes `BackgroundColor` on an entity that also has an `InteractionPalette` has the same
problem** -- that is the one migration hazard for `civilization`.

**Phase 5 — positioning for `.bsn` assets**

Nothing to build yet; the shape is already right. Widgets are free functions with plain
parameters, colors resolve through tokens rather than being captured, and the only asset
handles in play (`Handle<Font>`) go through the theme rather than the scene. When a
`.bsn` loader lands, the widgets are referenceable as-is.
