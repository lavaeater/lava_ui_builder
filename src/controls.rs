//! Themed form controls, built on Bevy's headless widgets (`bevy::ui_widgets`).
//!
//! Bevy supplies the behaviour -- dragging, keyboard handling, checked state, text
//! editing with selection and clipboard. This module supplies the look (all of it theme
//! tokens) and the game-shaped conveniences:
//!
//! | scene                | emits                 | state lives in            |
//! |----------------------|-----------------------|---------------------------|
//! | [`slider`]           | `ValueChange<f32>`    | `SliderValue`             |
//! | [`checkbox`]/[`toggle`] | `ValueChange<bool>` | `Checked` marker          |
//! | [`choice`]           | `ValueChange<usize>`  | `Checked` on the option   |
//! | [`cycle`]            | `ValueChange<usize>`  | [`Cycle::index`]          |
//! | [`select`]           | `ValueChange<usize>`  | [`Select::index`]         |
//! | [`text_input`]/[`text_area`] | `TextEditChange` | `EditableText`           |
//!
//! Every control updates itself (no app code needed to make it *work*), so a handler only
//! has to react: `on(|v: On<ValueChange<f32>>, mut a: ResMut<Audio>| a.volume = v.value)`.
//! To skip even that, bind it to a resource with [`crate::bind`].

use bevy::input_focus::InputFocus;
use bevy::picking::events::PointerPress;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, Justify, LineBreak, TextCursorStyle, TextLayout};
use bevy::ui::{Checked, Selected};
use bevy::ui_widgets::{
    Activate, Button as WidgetsButton, Checkbox, RadioButton, RadioGroup, Slider, SliderRange,
    SliderStep, SliderThumb, SliderValue, TextInput, TrackClick, ValueChange,
    checkbox_self_update, radio_self_update, slider_self_update,
};

use crate::scenes::{focusable, label, list_item};
use crate::{
    ColorToken, FontToken, LavaTheme, ThemedBackground, ThemedBorderColor, ThemedFont,
    ThemedTextColor, Variant, VariantText,
};

/// Registers the systems and observers that make the controls work. Added by
/// [`crate::LavaUiPlugin`].
pub struct ControlsPlugin;

impl Plugin for ControlsPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(step_cycle_on_activate)
            .add_observer(step_cycle)
            .add_observer(select_on_activate)
            .add_observer(close_selects_on_outside_press)
            .add_systems(
                Update,
                (
                    sync_sliders,
                    sync_checks,
                    sync_cycle_labels,
                    sync_selects,
                    sync_text_inputs,
                ),
            );
    }
}

// ============================================================================
// Slider
// ============================================================================

const SLIDER_MIN_WIDTH: f32 = 200.0;
const SLIDER_HEIGHT: f32 = 28.0;
const SLIDER_TRACK: f32 = 8.0;
const SLIDER_THUMB: f32 = 22.0;

/// Marks a lava-drawn slider (the headless [`Slider`] plus track, fill and thumb).
#[derive(Component, Clone, Default)]
pub struct LavaSlider;

/// The filled part of a [`slider`]'s track.
#[derive(Component, Clone, Default)]
pub struct SliderFillBar;

/// A themed horizontal slider from `min` to `max`, starting at `value`. At least 200 px
/// wide; it stretches to fill a `card` or column (patch `Node { width: .. }` to fix it).
///
/// Arrow keys, D-pad and stick move it by one [`SliderStep`] (a tenth of the range by
/// default; patch `SliderStep(1.0)` for whole numbers). Click or drag anywhere on it.
/// Emits `ValueChange<f32>` and updates its own `SliderValue`.
pub fn slider(min: f32, max: f32, value: f32) -> impl Scene {
    let range = SliderRange::new(min, max);
    let step = ((max - min) / 10.0).abs().max(f32::EPSILON);
    let pct = range.thumb_position(value) * 100.0;
    let thumb_top = (SLIDER_HEIGHT - SLIDER_THUMB) / 2.0;
    bsn! {
        Node {
            min_width: {px(SLIDER_MIN_WIDTH)},
            height: {px(SLIDER_HEIGHT)},
            align_items: AlignItems::Center,
        }
        LavaSlider
        Slider { track_click: TrackClick::Snap }
        SliderValue({value})
        ~{range}
        SliderStep({step})
        Hovered
        @focusable()
        on(slider_self_update)
        Children [
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                height: {px(SLIDER_TRACK)},
                border_radius: BorderRadius::MAX,
                overflow: {Overflow::clip()},
            }
            ThemedBackground(ColorToken::Track)
            Pickable::IGNORE
            Children [
                SliderFillBar
                Node { width: {percent(pct)}, height: percent(100) }
                ThemedBackground(ColorToken::Fill)
                Pickable::IGNORE
            ]
            --
            // The thumb travels over the track minus its own width, so it never overhangs
            // the ends -- which is also what the headless slider's drag maths assumes.
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: {px(SLIDER_THUMB)},
                top: px(0),
                bottom: px(0),
            }
            Pickable::IGNORE
            Children [
                SliderThumb
                Node {
                    position_type: PositionType::Absolute,
                    left: {percent(pct)},
                    top: {px(thumb_top)},
                    width: {px(SLIDER_THUMB)},
                    height: {px(SLIDER_THUMB)},
                    border_radius: BorderRadius::MAX,
                }
                ThemedBackground(ColorToken::Thumb)
            ]
        ]
    }
}

/// Move a slider's fill and thumb to its value, and light the thumb while hovered.
#[expect(clippy::type_complexity, reason = "a Bevy query tuple")]
fn sync_sliders(
    sliders: Query<
        (Entity, &SliderValue, &SliderRange, &Hovered),
        (With<LavaSlider>, Or<(Changed<SliderValue>, Changed<Hovered>, Added<LavaSlider>)>),
    >,
    children: Query<&Children>,
    mut fills: Query<&mut Node, (With<SliderFillBar>, Without<SliderThumb>)>,
    mut thumbs: Query<(&mut Node, &mut ThemedBackground), With<SliderThumb>>,
) {
    for (slider, value, range, hovered) in &sliders {
        let pct = percent(range.thumb_position(value.0) * 100.0);
        for descendant in children.iter_descendants(slider) {
            if let Ok(mut node) = fills.get_mut(descendant)
                && node.width != pct
            {
                node.width = pct;
            }
            if let Ok((mut node, mut bg)) = thumbs.get_mut(descendant) {
                if node.left != pct {
                    node.left = pct;
                }
                let token = if hovered.get() {
                    ColorToken::ThumbHovered
                } else {
                    ColorToken::Thumb
                };
                if bg.0 != token {
                    bg.0 = token;
                }
            }
        }
    }
}

// ============================================================================
// Checkbox and toggle
// ============================================================================

/// How a [`Checkbox`] is drawn.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckStyle {
    /// A box with a mark inside: [`checkbox`].
    #[default]
    Box,
    /// A pill with a knob that slides across: [`toggle`].
    Switch,
}

/// The box (checkbox) or track (toggle) part of a check control.
#[derive(Component, Clone, Default)]
pub struct CheckFrame;

/// The mark (checkbox) or knob (toggle) part of a check control.
#[derive(Component, Clone, Default)]
pub struct CheckMarkPart;

/// A themed checkbox with a label; click anywhere on the row to toggle it. Emits
/// `ValueChange<bool>` and adds or removes Bevy's `Checked` marker itself.
pub fn checkbox(text: impl Into<String>, checked: bool) -> impl Scene {
    let text = text.into();
    (
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
            }
            Checkbox
            CheckStyle::Box
            Hovered
            @focusable()
            on(checkbox_self_update)
            Children [
                CheckFrame
                Node {
                    width: px(26),
                    height: px(26),
                    border: {UiRect::all(px(2.0))},
                    border_radius: {BorderRadius::all(px(6.0))},
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                }
                ThemedBackground(ColorToken::Track)
                ThemedBorderColor(ColorToken::InputBorder)
                Pickable::IGNORE
                Children [
                    CheckMarkPart
                    Node { width: px(12), height: px(12), border_radius: {BorderRadius::all(px(3.0))} }
                    ThemedBackground(ColorToken::CheckMark)
                    Visibility::Hidden
                    Pickable::IGNORE
                ]
                --
                @label(text)
                Pickable::IGNORE
            ]
        },
        checked.then_some(bsn! { Checked }),
    )
}

/// A themed on/off switch with a label. Same behaviour and events as [`checkbox`].
pub fn toggle(text: impl Into<String>, is_on: bool) -> impl Scene {
    let text = text.into();
    (
        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
            }
            Checkbox
            CheckStyle::Switch
            Hovered
            @focusable()
            on(checkbox_self_update)
            Children [
                CheckFrame
                Node {
                    width: px(50),
                    height: px(28),
                    padding: {UiRect::all(px(3.0))},
                    border_radius: BorderRadius::MAX,
                    align_items: AlignItems::Center,
                }
                ThemedBackground(ColorToken::Track)
                Pickable::IGNORE
                Children [
                    CheckMarkPart
                    Node { width: px(22), height: px(22), border_radius: BorderRadius::MAX }
                    ThemedBackground(ColorToken::Thumb)
                    Pickable::IGNORE
                ]
                --
                @label(text)
                Pickable::IGNORE
            ]
        },
        is_on.then_some(bsn! { Checked }),
    )
}

/// Draw each check control from its `Checked` state. Runs every frame rather than on
/// change, because `Checked` is removed to uncheck and removal is invisible to
/// `Changed`; every write is guarded.
#[expect(clippy::type_complexity, reason = "a Bevy query tuple")]
fn sync_checks(
    checks: Query<(Entity, &CheckStyle, Has<Checked>)>,
    children: Query<&Children>,
    mut frames: Query<(&mut Node, &mut ThemedBackground), (With<CheckFrame>, Without<CheckMarkPart>)>,
    mut marks: Query<&mut Visibility, With<CheckMarkPart>>,
) {
    for (entity, style, checked) in &checks {
        for descendant in children.iter_descendants(entity) {
            if let Ok((mut node, mut bg)) = frames.get_mut(descendant)
                && *style == CheckStyle::Switch
            {
                let (justify, token) = if checked {
                    (JustifyContent::FlexEnd, ColorToken::Fill)
                } else {
                    (JustifyContent::FlexStart, ColorToken::Track)
                };
                if node.justify_content != justify {
                    node.justify_content = justify;
                }
                if bg.0 != token {
                    bg.0 = token;
                }
            }
            if let Ok(mut visibility) = marks.get_mut(descendant) {
                let wanted = if checked || *style == CheckStyle::Switch {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *visibility != wanted {
                    *visibility = wanted;
                }
            }
        }
    }
}

// ============================================================================
// Choice: a row of mutually exclusive options (a radio group drawn as buttons)
// ============================================================================

/// The position of an option inside a [`choice`] or [`select`].
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OptionIndex(pub usize);

/// A row of mutually exclusive buttons -- quality Low / Medium / High, difficulty, team.
/// The chosen one carries `Checked` and is painted in the theme's *selected* colours.
///
/// Emits `ValueChange<usize>` (the option's index) on the choice entity. Each option is
/// its own focus stop, so left/right moves between them and up/down leaves the row.
pub fn choice(options: &[&str], selected: usize) -> impl Scene {
    let options: Vec<_> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let option = (*option).to_string();
            (
                bsn! {
                    RadioButton
                    OptionIndex({i})
                    Node {
                        height: px(44),
                        padding: {UiRect::horizontal(px(18.0))},
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: {UiRect::all(px(2.0))},
                        border_radius: {BorderRadius::all(px(8.0))},
                    }
                    Hovered
                    @focusable()
                    Variant::Secondary
                    ThemedBorderColor(ColorToken::ButtonBorder)
                    Children [
                        Text({option})
                        ThemedFont(FontToken::Label)
                        VariantText
                        Pickable::IGNORE
                    ]
                },
                (i == selected).then_some(bsn! { Checked }),
            )
        })
        .collect();
    bsn! {
        RadioGroup
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(6),
            row_gap: px(6),
        }
        on(radio_self_update)
        on(emit_choice_index)
        Children [ {options} ]
    }
}

/// Translate the radio group's `ValueChange<Entity>` (which option entity) into the
/// `ValueChange<usize>` (which index) that the rest of the controls speak.
fn emit_choice_index(
    change: On<ValueChange<Entity>>,
    options: Query<&OptionIndex>,
    mut commands: Commands,
) {
    let Ok(index) = options.get(change.value) else {
        return;
    };
    commands.trigger(ValueChange {
        source: change.source,
        value: index.0,
        is_final: true,
    });
}

// ============================================================================
// Cycle: "< Medium >"
// ============================================================================

/// A value picked by stepping through a list: [`cycle`].
#[derive(Component, Clone, Debug, Default)]
pub struct Cycle {
    pub options: Vec<String>,
    pub index: usize,
}

/// An arrow inside a [`cycle`]: activating it steps the cycle by this much.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct CycleArrow(pub i32);

/// The caption inside a [`cycle`].
#[derive(Component, Clone, Default)]
pub struct CycleCaption;

/// Step a [`Cycle`] (wrapping). Triggered by the arrows, and by left/right while the
/// cycle has focus.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct StepCycle {
    pub entity: Entity,
    pub delta: i32,
}

/// `< Medium >`: one focus stop that steps through `options` with left/right (keys,
/// D-pad, stick) or its arrow buttons, wrapping at the ends. The gamepad-friendly
/// alternative to a dropdown. Emits `ValueChange<usize>`.
pub fn cycle(options: &[&str], index: usize) -> impl Scene {
    let options: Vec<String> = options.iter().map(|o| (*o).to_string()).collect();
    let current = options.get(index).cloned().unwrap_or_default();
    bsn! {
        Cycle { options: {options}, index: {index} }
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: px(6),
            padding: {UiRect::all(px(3.0))},
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(10.0))},
        }
        ThemedBackground(ColorToken::InputBg)
        ThemedBorderColor(ColorToken::InputBorder)
        Hovered
        @focusable()
        Children [
            @cycle_arrow("<", -1)
            --
            CycleCaption
            Text({current})
            ThemedFont(FontToken::Label)
            ThemedTextColor(ColorToken::InputText)
            TextLayout { justify: Justify::Center }
            Node { min_width: px(130), flex_grow: 1.0 }
            Pickable::IGNORE
            --
            @cycle_arrow(">", 1)
        ]
    }
}

fn cycle_arrow(glyph: &'static str, delta: i32) -> impl Scene {
    bsn! {
        CycleArrow({delta})
        WidgetsButton
        Hovered
        Variant::Ghost
        Node {
            width: px(36),
            height: px(36),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: {BorderRadius::all(px(7.0))},
        }
        Children [
            Text({glyph})
            ThemedFont(FontToken::Label)
            VariantText
            Pickable::IGNORE
        ]
    }
}

fn step_cycle_on_activate(
    activate: On<Activate>,
    arrows: Query<(&CycleArrow, &ChildOf)>,
    mut commands: Commands,
) {
    if let Ok((arrow, child_of)) = arrows.get(activate.entity) {
        commands.trigger(StepCycle {
            entity: child_of.parent(),
            delta: arrow.0,
        });
    }
}

fn step_cycle(step: On<StepCycle>, mut cycles: Query<&mut Cycle>, mut commands: Commands) {
    let Ok(mut cycle) = cycles.get_mut(step.entity) else {
        return;
    };
    let len = cycle.options.len();
    if len == 0 {
        return;
    }
    let index = (cycle.index as i64 + i64::from(step.delta)).rem_euclid(len as i64) as usize;
    cycle.index = index;
    commands.trigger(ValueChange {
        source: step.entity,
        value: index,
        is_final: true,
    });
}

fn sync_cycle_labels(
    cycles: Query<(Entity, &Cycle), Changed<Cycle>>,
    children: Query<&Children>,
    mut captions: Query<&mut Text, With<CycleCaption>>,
) {
    for (entity, cycle) in &cycles {
        let Some(current) = cycle.options.get(cycle.index) else {
            continue;
        };
        for descendant in children.iter_descendants(entity) {
            if let Ok(mut text) = captions.get_mut(descendant)
                && text.0 != *current
            {
                text.0.clone_from(current);
            }
        }
    }
}

// ============================================================================
// Select: a dropdown
// ============================================================================

/// A dropdown: [`select`].
#[derive(Component, Clone, Debug, Default)]
pub struct Select {
    pub options: Vec<String>,
    pub index: usize,
    pub open: bool,
    /// `open` as of the last sync: focus only moves on an open/close *transition*, never
    /// because the index changed or the dropdown was just spawned.
    was_open: bool,
}

/// The always-visible button of a [`select`].
#[derive(Component, Clone, Default)]
pub struct SelectButton;

/// The caption on a [`select`]'s button.
#[derive(Component, Clone, Default)]
pub struct SelectCaption;

/// The popup list of a [`select`].
#[derive(Component, Clone, Default)]
pub struct SelectPopup;

/// A dropdown. The button shows the current option; activating it opens a list below,
/// over whatever is there. Picking an option (click, ENTER, gamepad A) closes it; so do
/// Escape / gamepad B and a click elsewhere. Emits `ValueChange<usize>`.
pub fn select(options: &[&str], index: usize) -> impl Scene {
    let rows: Vec<_> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let option = (*option).to_string();
            bsn! { @list_item(option, i == index) OptionIndex({i}) }
        })
        .collect();
    let options: Vec<String> = options.iter().map(|o| (*o).to_string()).collect();
    let current = options.get(index).cloned().unwrap_or_default();
    bsn! {
        Select { options: {options}, index: {index} }
        Node { flex_direction: FlexDirection::Column, min_width: px(220) }
        Children [
            SelectButton
            WidgetsButton
            Hovered
            @focusable()
            Variant::Secondary
            Node {
                height: px(44),
                padding: {UiRect::horizontal(px(14.0))},
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(12),
                border: {UiRect::all(px(2.0))},
                border_radius: {BorderRadius::all(px(8.0))},
            }
            ThemedBorderColor(ColorToken::InputBorder)
            Children [
                SelectCaption
                Text({current})
                ThemedFont(FontToken::Label)
                VariantText
                Pickable::IGNORE
                --
                // ASCII: Bevy's default font has no arrow glyphs.
                Text("v")
                ThemedFont(FontToken::Small)
                VariantText
                Pickable::IGNORE
            ]
            --
            SelectPopup
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: percent(100),
                left: px(0),
                min_width: percent(100),
                flex_direction: FlexDirection::Column,
                margin: {UiRect::top(px(4.0))},
                padding: {UiRect::all(px(4.0))},
                row_gap: px(2),
                border: {UiRect::all(px(1.0))},
                border_radius: {BorderRadius::all(px(8.0))},
            }
            Visibility::Hidden
            GlobalZIndex(200)
            ThemedBackground(ColorToken::SurfaceRaised)
            ThemedBorderColor(ColorToken::SurfaceBorder)
            Children [ {rows} ]
        ]
    }
}

fn select_on_activate(
    activate: On<Activate>,
    buttons: Query<(), With<SelectButton>>,
    options: Query<&OptionIndex>,
    parents: Query<&ChildOf>,
    mut selects: Query<&mut Select>,
    mut commands: Commands,
) {
    let target = activate.entity;
    let Some(select_entity) = parents.iter_ancestors(target).find(|e| selects.contains(*e)) else {
        return;
    };
    let Ok(mut select) = selects.get_mut(select_entity) else {
        return;
    };
    if buttons.contains(target) {
        select.open = !select.open;
    } else if let Ok(index) = options.get(target) {
        select.open = false;
        if select.index != index.0 {
            select.index = index.0;
            commands.trigger(ValueChange {
                source: select_entity,
                value: index.0,
                is_final: true,
            });
        }
    }
}

/// Close open dropdowns when the pointer presses anywhere outside them.
fn close_selects_on_outside_press(
    press: On<PointerPress>,
    parents: Query<&ChildOf>,
    mut selects: Query<(Entity, &mut Select)>,
) {
    // Global observers see every hop of a bubbling event; act on the original only.
    if press.entity != press.original_event_target() {
        return;
    }
    let inside: Vec<Entity> = std::iter::once(press.entity)
        .chain(parents.iter_ancestors(press.entity))
        .collect();
    for (entity, mut select) in &mut selects {
        if select.open && !inside.contains(&entity) {
            select.open = false;
        }
    }
}

/// Show or hide each dropdown's popup, keep its caption and selected row current, and
/// move focus into the list when it opens (and back to the button when it closes).
#[expect(clippy::too_many_arguments, reason = "a system; one query per part of the widget")]
fn sync_selects(
    mut selects: Query<(Entity, &mut Select), Changed<Select>>,
    children: Query<&Children>,
    mut popups: Query<(&mut Node, &mut Visibility), With<SelectPopup>>,
    mut captions: Query<&mut Text, With<SelectCaption>>,
    options: Query<(Entity, &OptionIndex, Has<Selected>)>,
    buttons: Query<(), With<SelectButton>>,
    focus: Option<ResMut<InputFocus>>,
    mut commands: Commands,
) {
    let mut focus = focus;
    for (entity, mut select) in &mut selects {
        let toggled = select.open != select.was_open;
        if toggled {
            // Through `bypass_change_detection`, or this write would re-trigger the sync.
            select.bypass_change_detection().was_open = select.open;
        }
        let mut focus_target = None;
        for descendant in children.iter_descendants(entity) {
            if let Ok((mut node, mut visibility)) = popups.get_mut(descendant) {
                let (display, vis) = if select.open {
                    (Display::Flex, Visibility::Inherited)
                } else {
                    (Display::None, Visibility::Hidden)
                };
                if node.display != display {
                    node.display = display;
                }
                if *visibility != vis {
                    *visibility = vis;
                }
            }
            if let Ok(mut text) = captions.get_mut(descendant)
                && let Some(current) = select.options.get(select.index)
                && text.0 != *current
            {
                text.0.clone_from(current);
            }
            if let Ok((option, index, selected)) = options.get(descendant) {
                let wanted = index.0 == select.index;
                if wanted && !selected {
                    commands.entity(option).insert(Selected);
                } else if !wanted && selected {
                    commands.entity(option).remove::<Selected>();
                }
                if wanted && select.open {
                    focus_target = Some(option);
                }
            }
            if !select.open && buttons.contains(descendant) {
                focus_target = Some(descendant);
            }
        }
        // Only steer focus that is already inside this dropdown (or nowhere): opening a
        // list with the mouse should not yank focus from elsewhere.
        if let (true, Some(target), Some(focus)) = (toggled, focus_target, focus.as_mut()) {
            let inside = focus.get().is_none_or(|f| {
                f == entity || children.iter_descendants(entity).any(|d| d == f)
            });
            if inside {
                focus.set(target, bevy::input_focus::FocusCause::Navigated);
            }
        }
    }
}

/// Close every open dropdown. Returns whether one was open -- the focus module uses it so
/// that Escape / gamepad B closes a dropdown before it navigates back a screen.
pub(crate) fn close_open_selects(selects: &mut Query<&mut Select>) -> bool {
    let mut closed = false;
    for mut select in selects.iter_mut() {
        if select.open {
            select.open = false;
            closed = true;
        }
    }
    closed
}

// ============================================================================
// Text input
// ============================================================================

/// Marks the editable entity inside a [`text_input`] / [`text_area`].
#[derive(Component, Clone, Default)]
pub struct LavaTextInput;

/// The placeholder shown while a [`text_input`] is empty.
#[derive(Component, Clone, Default)]
pub struct InputPlaceholder;

/// The focus ring for this entity is drawn around its parent instead: the text field's
/// frame rather than the bare editable text inside it.
#[derive(Component, Clone, Default)]
pub struct FocusRingOnParent;

/// A one-line themed text field showing `placeholder` while empty.
///
/// Editing is Bevy's: cursor keys, mouse placement, selection (shift / drag), copy, cut
/// and paste with the usual shortcuts, undo-free. Read the text with
/// [`input_text`] on the inner [`LavaTextInput`] entity's `EditableText`, or react to
/// `TextEditChange` on it.
pub fn text_input(placeholder: impl Into<String>) -> impl Scene {
    let placeholder = placeholder.into();
    bsn! {
        Node {
            min_width: px(280),
            height: px(46),
            padding: {UiRect::horizontal(px(12.0))},
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(8.0))},
            align_items: AlignItems::Center,
        }
        ThemedBackground(ColorToken::InputBg)
        ThemedBorderColor(ColorToken::InputBorder)
        Children [
            LavaTextInput
            TextInput
            FocusRingOnParent
            EditableText { cursor_width: 0.12 }
            TextLayout { linebreak: LineBreak::NoWrap }
            Node { flex_grow: 1.0 }
            @focusable()
            ThemedFont(FontToken::Label)
            ThemedTextColor(ColorToken::InputText)
            TextCursorStyle
            --
            InputPlaceholder
            Text({placeholder})
            ThemedFont(FontToken::Label)
            ThemedTextColor(ColorToken::Placeholder)
            Node { position_type: PositionType::Absolute, left: px(14) }
            Pickable::IGNORE
        ]
    }
}

/// A multi-line themed text field, `height` pixels tall; ENTER inserts a new line.
pub fn text_area(placeholder: impl Into<String>, height: f32) -> impl Scene {
    let placeholder = placeholder.into();
    bsn! {
        Node {
            min_width: px(280),
            height: {px(height)},
            padding: {UiRect::all(px(10.0))},
            border: {UiRect::all(px(2.0))},
            border_radius: {BorderRadius::all(px(8.0))},
            align_items: AlignItems::FlexStart,
        }
        ThemedBackground(ColorToken::InputBg)
        ThemedBorderColor(ColorToken::InputBorder)
        Children [
            LavaTextInput
            TextInput
            FocusRingOnParent
            EditableText { cursor_width: 0.12, allow_newlines: true }
            TextLayout { linebreak: LineBreak::WordBoundary }
            Node { flex_grow: 1.0, height: percent(100) }
            @focusable()
            ThemedFont(FontToken::Label)
            ThemedTextColor(ColorToken::InputText)
            TextCursorStyle
            --
            InputPlaceholder
            Text({placeholder})
            ThemedFont(FontToken::Label)
            ThemedTextColor(ColorToken::Placeholder)
            Node { position_type: PositionType::Absolute, left: px(12), top: px(10) }
            Pickable::IGNORE
        ]
    }
}

/// The current text of a [`text_input`] / [`text_area`].
pub fn input_text(editable: &EditableText) -> String {
    editable.value().to_string()
}

/// Theme the cursor and selection, and show the placeholder only while the field is
/// empty and unfocused.
fn sync_text_inputs(
    theme: Option<Res<LavaTheme>>,
    mut inputs: Query<(Entity, &EditableText, &ChildOf, &mut TextCursorStyle), With<LavaTextInput>>,
    children: Query<&Children>,
    mut placeholders: Query<&mut Visibility, With<InputPlaceholder>>,
    focus: Option<Res<InputFocus>>,
) {
    let focused = focus.and_then(|f| f.get());
    for (entity, editable, child_of, mut cursor) in &mut inputs {
        if let Some(theme) = theme.as_deref() {
            let c = &theme.controls;
            if cursor.color != c.cursor || cursor.selection_color != c.selection {
                cursor.color = c.cursor;
                cursor.selection_color = c.selection;
                cursor.unfocused_selection_color = c.selection.with_alpha(0.15);
            }
        }
        let show = editable.value() == "" && focused != Some(entity);
        let wanted = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        for sibling in children.iter_descendants(child_of.parent()) {
            if let Ok(mut visibility) = placeholders.get_mut(sibling)
                && *visibility != wanted
            {
                *visibility = wanted;
            }
        }
    }
}
