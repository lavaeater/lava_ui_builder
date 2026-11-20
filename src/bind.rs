//! Data binding: point a widget at game state and stop writing sync systems.
//!
//! Before:
//!
//! ```ignore
//! #[derive(Component, Default, Clone)] struct AmmoText;
//! fn sync_ammo(hud: Res<Hud>, mut q: Query<&mut Text, With<AmmoText>>) {
//!     if !hud.is_changed() { return }
//!     for mut t in &mut q { **t = format!("{} / {}", hud.ammo, hud.ammo_max) }
//! }
//! ```
//!
//! After:
//!
//! ```ignore
//! @scenes::label("") @bind_text(|h: &Hud| format!("{} / {}", h.ammo, h.ammo_max))
//! // ...and once, in the app:
//! app.bind_resource::<Hud>();
//! ```
//!
//! A binding is a component holding a plain `fn` (a closure that captures nothing), so it
//! is `Clone` and can live in a scene. It writes only when its source changed (or when the
//! binding is first spawned), so an idle HUD costs one change check per binding.
//!
//! **One-way** bindings read a source into a widget:
//!
//! | scene                  | source                     | writes            |
//! |------------------------|----------------------------|-------------------|
//! | [`bind_text`]          | resource `R`               | `Text`            |
//! | [`bind_text_to`]       | component `C` on an entity | `Text`            |
//! | [`bind_meter`]         | resource `R`               | `ProgressBar`     |
//! | [`bind_meter_to`]      | component `C` on an entity | `ProgressBar`     |
//! | [`bind_visible`]       | resource `R`               | `Node::display`   |
//!
//! **Two-way** bindings also write the control's changes back into the resource, which is
//! what a settings screen is made of:
//!
//! | scene            | control                          | value   |
//! |------------------|----------------------------------|---------|
//! | [`bind_slider`]  | `scenes::slider`                 | `f32`   |
//! | [`bind_checked`] | `scenes::checkbox` / `toggle`    | `bool`  |
//! | [`bind_choice`]  | `scenes::choice` / `cycle` / `select` | `usize` |
//! | [`bind_input`]   | `scenes::text_input` / `text_area` | `&str` (write-back only) |
//!
//! Register each source type once with [`BindAppExt::bind_resource`] or
//! [`BindAppExt::bind_component`]. A binding whose type was never registered does
//! nothing; registering twice is harmless.

use std::any::TypeId;

use bevy::ecs::component::Mutable;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::text::{EditableText, TextEditChange};
use bevy::ui::Checked;
use bevy::ui_widgets::{SliderValue, ValueChange};

use crate::ProgressBar;
use crate::controls::{Cycle, OptionIndex, Select};

macro_rules! fn_component {
    ($(#[$meta:meta])* $name:ident<$p:ident: $bound:ident> { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Component)]
        pub struct $name<$p: $bound> {
            $(pub $field: $ty),*
        }
        // Hand-written: a derive would demand `$p: Clone` / `$p: Default`, but only the
        // fields (fn pointers and entities) need to be.
        impl<$p: $bound> Clone for $name<$p> {
            fn clone(&self) -> Self {
                Self { $($field: self.$field),* }
            }
        }
        impl<$p: $bound> Default for $name<$p> {
            fn default() -> Self {
                Self { $($field: $default),* }
            }
        }
    };
}

fn_component! {
    /// Sets `Text` from resource `R`. See [`bind_text`].
    BindText<R: Resource> { get: fn(&R) -> String = |_| String::new() }
}
fn_component! {
    /// Sets `Text` from component `C` on `source`. See [`bind_text_to`].
    BindTextTo<C: Component> {
        source: Entity = Entity::PLACEHOLDER,
        get: fn(&C) -> String = |_| String::new(),
    }
}
fn_component! {
    /// Sets [`ProgressBar::value`] from resource `R`. See [`bind_meter`].
    BindMeter<R: Resource> { get: fn(&R) -> f32 = |_| 0.0 }
}
fn_component! {
    /// Sets [`ProgressBar::value`] from component `C` on `source`. See [`bind_meter_to`].
    BindMeterTo<C: Component> {
        source: Entity = Entity::PLACEHOLDER,
        get: fn(&C) -> f32 = |_| 0.0,
    }
}
fn_component! {
    /// Shows (`Display::Flex`) or hides (`Display::None`) from resource `R`.
    BindVisible<R: Resource> { get: fn(&R) -> bool = |_| true }
}
fn_component! {
    /// Two-way: a slider's value and a `f32` in resource `R`. See [`bind_slider`].
    BindSlider<R: Resource> {
        get: fn(&R) -> f32 = |_| 0.0,
        set: fn(&mut R, f32) = |_, _| {},
    }
}
fn_component! {
    /// Two-way: a checkbox / toggle and a `bool` in resource `R`. See [`bind_checked`].
    BindChecked<R: Resource> {
        get: fn(&R) -> bool = |_| false,
        set: fn(&mut R, bool) = |_, _| {},
    }
}
fn_component! {
    /// Two-way: a choice / cycle / select and a `usize` in resource `R`. See [`bind_choice`].
    BindChoice<R: Resource> {
        get: fn(&R) -> usize = |_| 0,
        set: fn(&mut R, usize) = |_, _| {},
    }
}

fn_component! {
    /// Write-back: a text field's contents into resource `R`. See [`bind_input`].
    BindInput<R: Resource> { set: fn(&mut R, &str) = |_, _| {} }
}

/// Keep this entity's `Text` equal to `get(&R)`.
///
/// ```ignore
/// @scenes::label("") @bind_text(|s: &Score| format!("{} - {}", s.red, s.blue))
/// ```
pub fn bind_text<R: Resource>(get: fn(&R) -> String) -> impl Scene {
    let binding = BindText { get };
    bsn! { ~{binding} }
}

/// Keep this entity's `Text` equal to `get(&C)` for the `C` on `source` -- a nameplate
/// showing its character's health, say.
pub fn bind_text_to<C: Component>(source: Entity, get: fn(&C) -> String) -> impl Scene {
    let binding = BindTextTo { source, get };
    bsn! { ~{binding} }
}

/// Keep this meter's value equal to `get(&R)` (a fraction, `0.0..=1.0`).
pub fn bind_meter<R: Resource>(get: fn(&R) -> f32) -> impl Scene {
    let binding = BindMeter { get };
    bsn! { ~{binding} }
}

/// Keep this meter's value equal to `get(&C)` for the `C` on `source`.
///
/// ```ignore
/// @scenes::meter(1.0, 110.0, 12.0, ColorToken::HealthFill)
/// @bind_meter_to(character, |h: &Health| h.current / h.max)
/// ```
pub fn bind_meter_to<C: Component>(source: Entity, get: fn(&C) -> f32) -> impl Scene {
    let binding = BindMeterTo { source, get };
    bsn! { ~{binding} }
}

/// Show this entity while `get(&R)` is true, collapse it (`Display::None`) otherwise.
pub fn bind_visible<R: Resource>(get: fn(&R) -> bool) -> impl Scene {
    let binding = BindVisible { get };
    bsn! { ~{binding} }
}

/// Two-way bind a `scenes::slider` to a value in `R`:
///
/// ```ignore
/// @scenes::slider(0.0, 1.0, 0.7) @bind_slider(|a: &Audio| a.volume, |a, v| a.volume = v)
/// ```
pub fn bind_slider<R: Resource>(get: fn(&R) -> f32, set: fn(&mut R, f32)) -> impl Scene {
    let binding = BindSlider { get, set };
    bsn! { ~{binding} }
}

/// Two-way bind a `scenes::checkbox` / `scenes::toggle` to a flag in `R`.
pub fn bind_checked<R: Resource>(get: fn(&R) -> bool, set: fn(&mut R, bool)) -> impl Scene {
    let binding = BindChecked { get, set };
    bsn! { ~{binding} }
}

/// Two-way bind a `scenes::choice` / `cycle` / `select` to an index in `R`. Map an enum
/// with a small `as usize` / `from_index` pair:
///
/// ```ignore
/// @scenes::choice(&["Low", "Medium", "High"], 1)
/// @bind_choice(|g: &Graphics| g.quality as usize, |g, i| g.quality = Quality::from_index(i))
/// ```
pub fn bind_choice<R: Resource>(get: fn(&R) -> usize, set: fn(&mut R, usize)) -> impl Scene {
    let binding = BindChoice { get, set };
    bsn! { ~{binding} }
}

/// Write a `scenes::text_input` / `text_area`'s text into `R` as the player types. Patch
/// it onto the field itself:
///
/// ```ignore
/// @scenes::text_input("Title") @bind_input(|r: &mut Report, text| r.title = text.to_string())
/// ```
///
/// Write-back only: the field starts empty rather than showing `R`'s current value.
pub fn bind_input<R: Resource>(set: fn(&mut R, &str)) -> impl Scene {
    let binding = BindInput { set };
    bsn! { ~{binding} }
}

/// Which source types already have their binding systems.
#[derive(Resource, Default)]
struct BoundTypes(HashSet<TypeId>);

/// Registers the systems behind the `bind_*` scenes, once per source type.
pub trait BindAppExt {
    /// Make bindings that read resource `R` work: [`bind_text`], [`bind_meter`],
    /// [`bind_visible`], [`bind_slider`], [`bind_checked`], [`bind_choice`],
    /// [`bind_input`].
    fn bind_resource<R: Resource + Component<Mutability = Mutable>>(&mut self) -> &mut Self;
    /// Make bindings that read component `C` on another entity work: [`bind_text_to`],
    /// [`bind_meter_to`].
    fn bind_component<C: Component>(&mut self) -> &mut Self;
}

impl BindAppExt for App {
    fn bind_resource<R: Resource + Component<Mutability = Mutable>>(&mut self) -> &mut Self {
        if !first_registration::<R>(self) {
            return self;
        }
        self.add_systems(
            Update,
            (
                sync_resource_text::<R>,
                sync_resource_meter::<R>,
                sync_resource_visible::<R>,
                sync_resource_slider::<R>,
                sync_resource_checked::<R>,
                sync_resource_choice::<R>,
            )
                .run_if(resource_exists::<R>),
        )
        .add_observer(write_back_slider::<R>)
        .add_observer(write_back_checked::<R>)
        .add_observer(write_back_choice::<R>)
        .add_observer(write_back_input::<R>)
    }

    fn bind_component<C: Component>(&mut self) -> &mut Self {
        if !first_registration::<C>(self) {
            return self;
        }
        self.add_systems(Update, (sync_component_text::<C>, sync_component_meter::<C>))
    }
}

fn first_registration<T: 'static>(app: &mut App) -> bool {
    app.world_mut()
        .get_resource_or_insert_with(BoundTypes::default)
        .0
        .insert(TypeId::of::<T>())
}

// ── one-way, from a resource ────────────────────────────────────────────────────────

fn sync_resource_text<R: Resource>(source: Res<R>, mut q: Query<(Ref<BindText<R>>, &mut Text)>) {
    for (binding, mut text) in &mut q {
        if source.is_changed() || binding.is_added() {
            let value = (binding.get)(&source);
            if text.0 != value {
                text.0 = value;
            }
        }
    }
}

fn sync_resource_meter<R: Resource>(
    source: Res<R>,
    mut q: Query<(Ref<BindMeter<R>>, &mut ProgressBar)>,
) {
    for (binding, mut bar) in &mut q {
        if source.is_changed() || binding.is_added() {
            let value = (binding.get)(&source).clamp(0.0, 1.0);
            if (bar.value - value).abs() > f32::EPSILON {
                bar.value = value;
            }
        }
    }
}

fn sync_resource_visible<R: Resource>(
    source: Res<R>,
    mut q: Query<(Ref<BindVisible<R>>, &mut Node)>,
) {
    for (binding, mut node) in &mut q {
        if source.is_changed() || binding.is_added() {
            let display = if (binding.get)(&source) {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }
    }
}

// ── two-way ─────────────────────────────────────────────────────────────────────────

fn sync_resource_slider<R: Resource>(
    source: Res<R>,
    q: Query<(Entity, Ref<BindSlider<R>>, &SliderValue)>,
    mut commands: Commands,
) {
    for (entity, binding, current) in &q {
        if source.is_changed() || binding.is_added() {
            let value = (binding.get)(&source);
            if (current.0 - value).abs() > f32::EPSILON {
                // `SliderValue` is immutable: replacing it is how its value changes.
                commands.entity(entity).insert(SliderValue(value));
            }
        }
    }
}

fn write_back_slider<R: Resource + Component<Mutability = Mutable>>(
    change: On<ValueChange<f32>>,
    bindings: Query<&BindSlider<R>>,
    source: Option<ResMut<R>>,
) {
    if let (Ok(binding), Some(mut source)) = (bindings.get(change.source), source) {
        (binding.set)(&mut *source, change.value);
    }
}

fn sync_resource_checked<R: Resource>(
    source: Res<R>,
    q: Query<(Entity, Ref<BindChecked<R>>, Has<Checked>)>,
    mut commands: Commands,
) {
    for (entity, binding, checked) in &q {
        if source.is_changed() || binding.is_added() {
            let wanted = (binding.get)(&source);
            if wanted && !checked {
                commands.entity(entity).insert(Checked);
            } else if !wanted && checked {
                commands.entity(entity).remove::<Checked>();
            }
        }
    }
}

fn write_back_checked<R: Resource + Component<Mutability = Mutable>>(
    change: On<ValueChange<bool>>,
    bindings: Query<&BindChecked<R>>,
    source: Option<ResMut<R>>,
) {
    if let (Ok(binding), Some(mut source)) = (bindings.get(change.source), source) {
        (binding.set)(&mut *source, change.value);
    }
}

/// Push the bound index into whichever kind of choice control this is.
#[expect(clippy::type_complexity, reason = "Bevy query tuples")]
fn sync_resource_choice<R: Resource>(
    source: Res<R>,
    bindings: Query<(Entity, Ref<BindChoice<R>>, Option<&Children>)>,
    mut cycles: Query<&mut Cycle>,
    mut selects: Query<&mut Select>,
    options: Query<(Entity, &OptionIndex, Has<Checked>), Without<Select>>,
    mut commands: Commands,
) {
    for (entity, binding, children) in &bindings {
        if !(source.is_changed() || binding.is_added()) {
            continue;
        }
        let index = (binding.get)(&source);
        if let Ok(mut cycle) = cycles.get_mut(entity) {
            if cycle.index != index {
                cycle.index = index;
            }
        } else if let Ok(mut select) = selects.get_mut(entity) {
            if select.index != index {
                select.index = index;
            }
        } else if let Some(children) = children {
            // A `choice`: its direct children are the options.
            for (option, option_index, checked) in children.iter().filter_map(|c| options.get(c).ok()) {
                let wanted = option_index.0 == index;
                if wanted && !checked {
                    commands.entity(option).insert(Checked);
                } else if !wanted && checked {
                    commands.entity(option).remove::<Checked>();
                }
            }
        }
    }
}

fn write_back_choice<R: Resource + Component<Mutability = Mutable>>(
    change: On<ValueChange<usize>>,
    bindings: Query<&BindChoice<R>>,
    source: Option<ResMut<R>>,
) {
    if let (Ok(binding), Some(mut source)) = (bindings.get(change.source), source) {
        (binding.set)(&mut *source, change.value);
    }
}

/// The edit lands on the inner editable entity; the binding sits on the field (its
/// parent) or on the editable itself.
fn write_back_input<R: Resource + Component<Mutability = Mutable>>(
    change: On<TextEditChange>,
    texts: Query<&EditableText>,
    bindings: Query<&BindInput<R>>,
    parents: Query<&ChildOf>,
    source: Option<ResMut<R>>,
) {
    let editable = change.event_target();
    let (Ok(text), Some(mut source)) = (texts.get(editable), source) else {
        return;
    };
    let binding = std::iter::once(editable)
        .chain(parents.iter_ancestors(editable))
        .find_map(|e| bindings.get(e).ok());
    if let Some(binding) = binding {
        (binding.set)(&mut *source, &text.value().to_string());
    }
}

// ── one-way, from a component on another entity ─────────────────────────────────────

fn sync_component_text<C: Component>(
    mut q: Query<(Ref<BindTextTo<C>>, &mut Text)>,
    sources: Query<Ref<C>>,
) {
    for (binding, mut text) in &mut q {
        let Ok(source) = sources.get(binding.source) else {
            continue;
        };
        if source.is_changed() || binding.is_added() {
            let value = (binding.get)(&source);
            if text.0 != value {
                text.0 = value;
            }
        }
    }
}

fn sync_component_meter<C: Component>(
    mut q: Query<(Ref<BindMeterTo<C>>, &mut ProgressBar)>,
    sources: Query<Ref<C>>,
) {
    for (binding, mut bar) in &mut q {
        let Ok(source) = sources.get(binding.source) else {
            continue;
        };
        if source.is_changed() || binding.is_added() {
            let value = (binding.get)(&source).clamp(0.0, 1.0);
            if (bar.value - value).abs() > f32::EPSILON {
                bar.value = value;
            }
        }
    }
}
