//! Keyboard and controller navigation, and the themed focus ring.
//!
//! Every interactive lava widget is focusable (`scenes::focusable()`: a `TabIndex` plus
//! Bevy's `AutoDirectionalNavigation`). This module drives that focus:
//!
//! | input                              | does                                          |
//! |------------------------------------|-----------------------------------------------|
//! | Tab / Shift+Tab                    | next / previous, in `TabIndex` order          |
//! | arrows, D-pad, left stick          | move to the nearest widget in that direction  |
//! | left/right on a slider or cycle    | change its value instead                      |
//! | ENTER / SPACE                      | activate (Bevy's widgets handle these)        |
//! | gamepad A                          | activate, toggle or pick the focused widget   |
//! | Escape, gamepad B, a `Back` button | close a dropdown, else cancel a dialog, else [`NavigateBack`] |
//!
//! Tab follows the same [`UiNavigation`] rules as the arrows: with nothing focused it does
//! nothing (unless `start_from_nothing`), so a game can bind Tab to something else while a
//! HUD is up. Tab order comes from Bevy's `TabGroup`s; `ui_root`, `screen`, `fullscreen`
//! and `anchored` are groups, and a `confirm` dialog is a modal one (Tab stays inside it).
//! Bevy's own `TabNavigationPlugin` is not used: it grabs focus from nothing and warns on
//! every Tab when a screen has no `TabGroup`. If the app adds it anyway (feathers does),
//! lava leaves Tab to it.
//!
//! The focused widget gets an `Outline` in [`crate::ColorToken::FocusRing`] while focus
//! is "visible" -- after keyboard or gamepad input, not after a mouse click, the same rule
//! browsers follow. A focused widget inside a scrolling list is scrolled into view.
//!
//! The ring is an `Outline` the module owns: an entity that should keep an outline of its
//! own will lose it while focused.

use bevy::input::ButtonState;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::directional_navigation::DirectionalNavigationPlugin;
use bevy::input_focus::tab_navigation::{
    NavAction, TabNavigation, TabNavigationError, TabNavigationPlugin,
};
use bevy::input_focus::{FocusCause, FocusedInput, InputFocus, InputFocusVisible};
use bevy::math::CompassOctant;
use bevy::picking::events::PointerPress;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::ui::auto_directional_navigation::{AutoDirectionalNavigation, AutoDirectionalNavigator};
use bevy::ui_widgets::{
    Activate, Button as WidgetsButton, Checkbox, RadioButton, SetSliderValue, SliderValueChange,
    TextInput, ToggleChecked, ValueChange,
};

use crate::LavaTheme;
use crate::controls::{
    Cycle, FocusRingOnParent, LavaSlider, Select, StepCycle, close_open_selects,
};
use crate::overlay::{Dialog, cancel_top_dialog};
use crate::screens::Back;

/// "Go back one level": the current screen's parent, if one is registered with
/// `screen_back`. Triggered by Escape, gamepad B and [`Back`] buttons -- unless a dropdown
/// or dialog was open, in which case that closes instead. Trigger it yourself to go back
/// from code.
#[derive(Event, Clone, Copy, Debug, Default)]
pub struct NavigateBack;

/// How keyboard / controller navigation behaves.
///
/// Games that steer with the arrow keys or the stick while a HUD is up should care about
/// `start_from_nothing`: when it is on, a direction pressed while *nothing* has focus
/// grabs the first widget on screen. That is off by default -- menus give their first
/// button `AutoFocus` instead -- so in-game movement never wakes the UI. The same goes for
/// Tab. Set `enabled: false` to switch all of it off.
#[derive(Resource, Clone, Copy, Debug)]
pub struct UiNavigation {
    pub enabled: bool,
    pub start_from_nothing: bool,
}

impl Default for UiNavigation {
    fn default() -> Self {
        Self {
            enabled: true,
            start_from_nothing: false,
        }
    }
}

/// Marks the entity currently wearing the focus ring.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct FocusRing;

/// The navigation systems and the focus ring. Added by [`crate::LavaUiPlugin`].
pub struct FocusPlugin;

impl Plugin for FocusPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<DirectionalNavigationPlugin>() {
            app.add_plugins(DirectionalNavigationPlugin);
        }
        app.init_resource::<StickLatch>()
            .init_resource::<UiNavigation>()
            .add_observer(back_button)
            .add_observer(hide_focus_ring_on_click)
            .add_systems(
                Update,
                (navigate, draw_focus_ring, scroll_focus_into_view).chain(),
            );
    }

    // In `finish`, so plugins added after this one (feathers) are seen too.
    fn finish(&self, app: &mut App) {
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_observer(tab_navigation);
        }
    }
}

/// Tab / Shift+Tab: Bevy's `handle_tab_navigation`, under [`UiNavigation`]'s rules and
/// without its warnings (a screen with no focusable widgets is normal in a game).
fn tab_navigation(
    mut event: On<FocusedInput<KeyboardInput>>,
    settings: Res<UiNavigation>,
    nav: TabNavigation,
    mut focus: ResMut<InputFocus>,
    visible: Option<ResMut<InputFocusVisible>>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
) {
    let key = &event.input;
    if key.key_code != KeyCode::Tab || key.state != ButtonState::Pressed || key.repeat {
        return;
    }
    if !settings.enabled || (focus.get().is_none() && !settings.start_from_nothing) {
        return;
    }
    let shift = keys.is_some_and(|k| k.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]));
    let action = if shift { NavAction::Previous } else { NavAction::Next };
    // Focus outside any group (a widget the app spawned on its own) still moves on.
    let (Ok(next) | Err(TabNavigationError::NoTabGroupForCurrentFocus { new_focus: next, .. })) =
        nav.navigate(&focus, action)
    else {
        return;
    };
    event.propagate(false);
    focus.set(next, FocusCause::Navigated);
    if let Some(mut visible) = visible {
        visible.0 = true;
    }
}

/// Remembers whether the stick was already pushed, so holding it moves focus once rather
/// than every frame.
#[derive(Resource, Default)]
struct StickLatch(bool);

/// One frame of navigation intent, from keyboard and every gamepad together.
#[derive(Default)]
struct Intent {
    /// From arrow keys.
    key_direction: Option<CompassOctant>,
    /// From D-pad or stick.
    pad_direction: Option<CompassOctant>,
    activate: bool,
    back_key: bool,
    back_pad: bool,
}

fn read_intent(
    keys: Option<&ButtonInput<KeyCode>>,
    gamepads: &Query<&Gamepad>,
    latch: &mut StickLatch,
) -> Intent {
    let mut intent = Intent::default();
    if let Some(keys) = keys {
        intent.key_direction = [
            (KeyCode::ArrowUp, CompassOctant::North),
            (KeyCode::ArrowDown, CompassOctant::South),
            (KeyCode::ArrowLeft, CompassOctant::West),
            (KeyCode::ArrowRight, CompassOctant::East),
        ]
        .into_iter()
        .find_map(|(key, dir)| keys.just_pressed(key).then_some(dir));
        intent.back_key = keys.just_pressed(KeyCode::Escape);
    }
    let mut stick_pushed = false;
    for gamepad in gamepads {
        let dpad = [
            (GamepadButton::DPadUp, CompassOctant::North),
            (GamepadButton::DPadDown, CompassOctant::South),
            (GamepadButton::DPadLeft, CompassOctant::West),
            (GamepadButton::DPadRight, CompassOctant::East),
        ]
        .into_iter()
        .find_map(|(button, dir)| gamepad.just_pressed(button).then_some(dir));
        let stick = gamepad.left_stick();
        let stick_dir = if stick.length() > 0.6 {
            stick_pushed = true;
            Some(if stick.x.abs() > stick.y.abs() {
                if stick.x > 0.0 {
                    CompassOctant::East
                } else {
                    CompassOctant::West
                }
            } else if stick.y > 0.0 {
                CompassOctant::North
            } else {
                CompassOctant::South
            })
        } else {
            None
        };
        if intent.pad_direction.is_none() {
            intent.pad_direction = dpad.or(if latch.0 { None } else { stick_dir });
        }
        intent.activate |= gamepad.just_pressed(GamepadButton::South);
        intent.back_pad |= gamepad.just_pressed(GamepadButton::East);
    }
    latch.0 = stick_pushed;
    intent
}

const fn horizontal(direction: CompassOctant) -> Option<i32> {
    match direction {
        CompassOctant::West => Some(-1),
        CompassOctant::East => Some(1),
        _ => None,
    }
}

/// What kind of widget the focused entity is, for the input rules above.
struct FocusedKind {
    is_slider: bool,
    is_cycle: bool,
    is_text: bool,
    is_button: bool,
    is_checkbox: bool,
    is_radio: bool,
}

#[expect(clippy::type_complexity, reason = "Bevy query tuples")]
#[expect(clippy::too_many_arguments, reason = "a system; each parameter is one input source")]
fn navigate(
    settings: Res<UiNavigation>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mut latch: ResMut<StickLatch>,
    mut navigator: AutoDirectionalNavigator,
    mut visible: Option<ResMut<InputFocusVisible>>,
    kinds: Query<(
        Has<LavaSlider>,
        Has<Cycle>,
        Has<TextInput>,
        Has<WidgetsButton>,
        Has<Checkbox>,
        Has<RadioButton>,
        Option<&ChildOf>,
    )>,
    candidates: Query<
        (Entity, &ComputedNode, &UiGlobalTransform, &InheritedVisibility),
        With<AutoDirectionalNavigation>,
    >,
    mut selects: Query<&mut Select>,
    dialogs: Query<Entity, With<Dialog>>,
    mut commands: Commands,
) {
    if !settings.enabled {
        return;
    }
    let intent = read_intent(keys.as_deref(), &gamepads, &mut latch);
    let focused = navigator.input_focus().filter(|e| kinds.contains(*e));
    let kind = focused.and_then(|e| kinds.get(e).ok()).map(
        |(is_slider, is_cycle, is_text, is_button, is_checkbox, is_radio, parent)| {
            (
                FocusedKind {
                    is_slider,
                    is_cycle,
                    is_text,
                    is_button,
                    is_checkbox,
                    is_radio,
                },
                parent.map(ChildOf::parent),
            )
        },
    );

    // While typing, the keyboard belongs to the text field.
    let typing = kind.as_ref().is_some_and(|(k, _)| k.is_text);
    let key_direction = if typing { None } else { intent.key_direction };

    let mut used_input = false;
    if let (Some(entity), Some((kind, _))) = (focused, kind.as_ref()) {
        // Left/right adjusts a slider or cycle in place instead of moving focus. Bevy's
        // slider already handles the arrow *keys*; the pad is ours.
        if let Some(delta) = intent.pad_direction.and_then(horizontal)
            && (kind.is_slider || kind.is_cycle)
        {
            if kind.is_slider {
                commands.trigger(SetSliderValue {
                    entity,
                    change: SliderValueChange::RelativeStep(delta as f32),
                });
            } else {
                commands.trigger(StepCycle { entity, delta });
            }
            used_input = true;
        } else if let Some(delta) = key_direction.and_then(horizontal)
            && (kind.is_slider || kind.is_cycle)
        {
            if kind.is_cycle {
                commands.trigger(StepCycle { entity, delta });
            }
            used_input = true;
        }
    }

    if !used_input && let Some(direction) = intent.pad_direction.or(key_direction) {
        if focused.is_some() {
            // No neighbour that way is fine: focus just stays put.
            let _ = navigator.navigate(direction);
        } else if settings.start_from_nothing
            && let Some(first) = first_candidate(&candidates)
        {
            navigator
                .manual_directional_navigation
                .focus
                .set(first, FocusCause::Navigated);
        }
        if let Some(visible) = visible.as_mut() {
            visible.0 = true;
        }
    }

    if intent.activate
        && let (Some(entity), Some((kind, parent))) = (focused, kind.as_ref())
    {
        if kind.is_button {
            commands.trigger(Activate { entity });
        } else if kind.is_checkbox {
            commands.trigger(ToggleChecked { entity });
        } else if kind.is_radio
            && let Some(group) = parent
        {
            commands.trigger(ValueChange {
                source: *group,
                value: entity,
                is_final: true,
            });
        } else if kind.is_cycle {
            commands.trigger(StepCycle { entity, delta: 1 });
        }
    }

    // Escape inside a text field only leaves the field (Bevy blurs it).
    if intent.back_pad || (intent.back_key && !typing) {
        go_back_one_level(&mut selects, &dialogs, &mut commands);
    }
}

/// The top-left-most visible focusable widget: where focus starts when there is none.
fn first_candidate(
    candidates: &Query<
        (Entity, &ComputedNode, &UiGlobalTransform, &InheritedVisibility),
        With<AutoDirectionalNavigation>,
    >,
) -> Option<Entity> {
    candidates
        .iter()
        .filter(|(_, node, _, visibility)| visibility.get() && node.size() != Vec2::ZERO)
        .min_by(|a, b| {
            let (pa, pb) = (a.2.translation, b.2.translation);
            (pa.y, pa.x)
                .partial_cmp(&(pb.y, pb.x))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(entity, ..)| entity)
}

/// The one rule for "back": close a dropdown, else cancel the top dialog, else navigate.
fn go_back_one_level(
    selects: &mut Query<&mut Select>,
    dialogs: &Query<Entity, With<Dialog>>,
    commands: &mut Commands,
) {
    if close_open_selects(selects) {
        return;
    }
    if cancel_top_dialog(dialogs, commands) {
        return;
    }
    commands.trigger(NavigateBack);
}

fn back_button(
    activate: On<Activate>,
    backs: Query<(), With<Back>>,
    mut selects: Query<&mut Select>,
    dialogs: Query<Entity, With<Dialog>>,
    mut commands: Commands,
) {
    if backs.contains(activate.entity) {
        go_back_one_level(&mut selects, &dialogs, &mut commands);
    }
}

/// A mouse click hides the ring (focus itself still moves, via Bevy's pointer focus).
fn hide_focus_ring_on_click(press: On<PointerPress>, visible: Option<ResMut<InputFocusVisible>>) {
    if press.entity != press.original_event_target() {
        return;
    }
    if let Some(mut visible) = visible
        && visible.0
    {
        visible.0 = false;
    }
}

/// Keep exactly one `Outline` + [`FocusRing`]: on the focused entity (or its parent, for
/// [`FocusRingOnParent`]) while focus is visible, nowhere otherwise.
fn draw_focus_ring(
    focus: Option<Res<InputFocus>>,
    visible: Option<Res<InputFocusVisible>>,
    theme: Option<Res<LavaTheme>>,
    mut rings: Query<(Entity, &mut Outline), With<FocusRing>>,
    targets: Query<(Has<FocusRingOnParent>, Option<&ChildOf>)>,
    mut commands: Commands,
) {
    let shown = visible.is_some_and(|v| v.0);
    let target = focus
        .and_then(|f| f.get())
        .filter(|_| shown)
        .and_then(|e| match targets.get(e) {
            Ok((true, Some(child_of))) => Some(child_of.parent()),
            Ok(_) => Some(e),
            Err(_) => None,
        });
    let color = theme.map_or(Color::WHITE, |t| t.controls.focus_ring);

    let mut has_ring = false;
    for (entity, mut outline) in &mut rings {
        if Some(entity) == target {
            has_ring = true;
            if outline.color != color {
                outline.color = color;
            }
        } else {
            commands.entity(entity).remove::<(Outline, FocusRing)>();
        }
    }
    if !has_ring && let Some(target) = target {
        commands
            .entity(target)
            .try_insert((Outline::new(px(3.0), px(3.0), color), FocusRing));
    }
}

/// Scroll the nearest scrolling ancestor so a newly focused widget is fully in view.
fn scroll_focus_into_view(
    focus: Option<Res<InputFocus>>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    parents: Query<&ChildOf>,
    mut scrollers: Query<(&mut ScrollPosition, &ComputedNode, &UiGlobalTransform, &Node)>,
) {
    let Some(focus) = focus.filter(DetectChanges::is_changed) else {
        return;
    };
    let Some(target) = focus.get() else {
        return;
    };
    let Ok((node, transform)) = nodes.get(target) else {
        return;
    };
    let Some(scroller) = parents.iter_ancestors(target).find(|e| {
        scrollers
            .get(*e)
            .is_ok_and(|(.., n)| n.overflow.y == OverflowAxis::Scroll)
    }) else {
        return;
    };
    let Ok((mut scroll, scroller_node, scroller_transform, _)) = scrollers.get_mut(scroller) else {
        return;
    };
    // Both rectangles in physical pixels; ScrollPosition is in logical ones.
    let half = node.size().y / 2.0;
    let (top, bottom) = (transform.translation.y - half, transform.translation.y + half);
    let view_half = scroller_node.size().y / 2.0;
    let (view_top, view_bottom) = (
        scroller_transform.translation.y - view_half,
        scroller_transform.translation.y + view_half,
    );
    let to_logical = scroller_node.inverse_scale_factor();
    if top < view_top {
        scroll.y -= (view_top - top) * to_logical;
    } else if bottom > view_bottom {
        scroll.y += (bottom - view_bottom) * to_logical;
    }
}
