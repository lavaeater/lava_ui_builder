//! Things drawn over the rest of the UI: tooltips, toasts and modal dialogs.
//!
//! * **Tooltips.** Patch `@scenes::tooltip("Diamond sword")` onto any entity. One shared
//!   tooltip follows the pointer while it hovers the entity, and sits under the entity
//!   while it has keyboard / gamepad focus.
//! * **Toasts.** `commands.trigger(Toast::success("Saved"))` stacks a short-lived
//!   notification in the bottom-right corner.
//! * **Dialogs.** `commands.spawn_scene(scenes::confirm("Quit?", "Unsaved progress is
//!   lost.", "Quit")).observe(on_answer)`: a modal over a dimmed backdrop. The observer gets
//!   a [`DialogResult`]; the dialog despawns itself.

use bevy::input_focus::{AutoFocus, InputFocus, InputFocusVisible};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::UiGlobalTransform;
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;

use crate::scenes::{backdrop, button, caption, card, header, row};
use crate::{ColorToken, FontToken, ThemedBackground, ThemedBorderColor, ThemedFont, ThemedTextColor, Variant};

/// Tooltips, toasts and dialogs. Added by [`crate::LavaUiPlugin`].
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(spawn_toast)
            .add_observer(answer_dialog)
            .add_systems(Update, (show_tooltip, expire_toasts));
    }
}

// ============================================================================
// Tooltip
// ============================================================================

/// Show this text in a tooltip while the entity is hovered or focused.
#[derive(Component, Clone, Debug, Default)]
#[require(Hovered)]
pub struct Tooltip(pub String);

/// Give an entity a tooltip: `@scenes::icon(..) @scenes::tooltip("Diamond sword")`.
pub fn tooltip(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! { Tooltip({text}) }
}

/// The single shared tooltip box.
#[derive(Component, Clone, Default)]
pub struct TooltipBox;

/// Its text.
#[derive(Component, Clone, Default)]
pub struct TooltipText;

fn tooltip_box() -> impl Scene {
    bsn! {
        TooltipBox
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            padding: {UiRect::axes(px(10.0), px(6.0))},
            border_radius: {BorderRadius::all(px(6.0))},
            max_width: px(320),
        }
        ThemedBackground(ColorToken::TooltipBg)
        GlobalZIndex(1000)
        Pickable::IGNORE
        Children [
            TooltipText
            Text("")
            ThemedFont(FontToken::Small)
            ThemedTextColor(ColorToken::TooltipText)
            Pickable::IGNORE
        ]
    }
}

/// Where the tooltip should be, in UI (logical, pre-`UiScale`) pixels, and what it says.
#[expect(clippy::too_many_arguments, reason = "a system; each parameter is one input")]
fn show_tooltip(
    tips: Query<(Entity, &Tooltip, &Hovered, &ComputedNode, &UiGlobalTransform)>,
    focus: Option<Res<InputFocus>>,
    focus_visible: Option<Res<InputFocusVisible>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Option<Res<UiScale>>,
    mut boxes: Query<(Entity, &mut Node), With<TooltipBox>>,
    children: Query<&Children>,
    mut texts: Query<&mut Text, With<TooltipText>>,
    mut commands: Commands,
) {
    let scale = ui_scale.map_or(1.0, |s| s.0).max(f32::EPSILON);
    let focused = focus
        .and_then(|f| f.get())
        .filter(|_| focus_visible.is_some_and(|v| v.0));
    let cursor = windows.single().ok().and_then(Window::cursor_position);

    // Hover wins over focus: the pointer is where the player is looking.
    let shown = tips
        .iter()
        .find(|(.., hovered, _, _)| hovered.get())
        .and_then(|(_, tip, ..)| cursor.map(|c| (tip.0.clone(), c / scale + Vec2::new(16.0, 20.0))))
        .or_else(|| {
            let (_, tip, _, node, transform) = tips.get(focused?).ok()?;
            let to_ui = node.inverse_scale_factor();
            let bottom_left = transform.translation - node.size() / 2.0 + Vec2::new(0.0, node.size().y);
            Some((tip.0.clone(), bottom_left * to_ui + Vec2::new(0.0, 8.0)))
        });

    let Some((tooltip, mut node)) = boxes.iter_mut().next() else {
        if shown.is_some() {
            commands.spawn_scene(tooltip_box());
        }
        return;
    };
    match shown {
        Some((text, at)) => {
            if node.display != Display::Flex {
                node.display = Display::Flex;
            }
            let (left, top) = (px(at.x.round()), px(at.y.round()));
            if node.left != left || node.top != top {
                node.left = left;
                node.top = top;
            }
            for child in children.iter_descendants(tooltip) {
                if let Ok(mut t) = texts.get_mut(child)
                    && t.0 != text
                {
                    t.0.clone_from(&text);
                }
            }
        }
        None => {
            if node.display != Display::None {
                node.display = Display::None;
            }
        }
    }
}

// ============================================================================
// Toasts
// ============================================================================

/// The tone of a [`Toast`]: picks the colour of its edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastKind {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl ToastKind {
    const fn token(self) -> ColorToken {
        match self {
            Self::Info => ColorToken::Accent,
            Self::Success => ColorToken::Success,
            Self::Warning => ColorToken::Warning,
            Self::Danger => ColorToken::Danger,
        }
    }
}

/// A short notification in the bottom-right corner: `commands.trigger(Toast::info("Saved"))`.
#[derive(Event, Clone, Debug)]
pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    /// How long it stays up.
    pub seconds: f32,
}

impl Toast {
    pub fn new(kind: ToastKind, text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind,
            seconds: 3.0,
        }
    }
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Info, text)
    }
    pub fn success(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Success, text)
    }
    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Warning, text)
    }
    pub fn danger(text: impl Into<String>) -> Self {
        Self::new(ToastKind::Danger, text)
    }
}

/// The corner column toasts stack in.
#[derive(Component, Clone, Default)]
pub struct ToastStack;

/// Time left on a toast.
#[derive(Component, Clone, Default)]
pub struct ToastLife(pub Timer);

fn spawn_toast(toast: On<Toast>, stacks: Query<Entity, With<ToastStack>>, mut commands: Commands) {
    let text = toast.text.clone();
    let token = toast.kind.token();
    let life = ToastLife(Timer::from_seconds(toast.seconds, TimerMode::Once));
    // A one-element `Vec` is a `SceneList`, which is what a `Children` slot takes.
    let item = vec![bsn! {
        ~{life}
        Node {
            padding: {UiRect::axes(px(14.0), px(10.0))},
            border: {UiRect::left(px(5.0))},
            border_radius: {BorderRadius::all(px(6.0))},
            max_width: px(360),
        }
        ThemedBackground(ColorToken::SurfaceRaised)
        ThemedBorderColor({token})
        Pickable::IGNORE
        Children [
            Text({text})
            ThemedFont(FontToken::Small)
            ThemedTextColor(ColorToken::LabelText)
            Pickable::IGNORE
        ]
    }];
    if let Some(stack) = stacks.iter().next() {
        commands
            .entity(stack)
            .queue_spawn_related_scenes::<Children>(item);
    } else {
        commands.spawn_scene(bsn! {
            ToastStack
            Node {
                position_type: PositionType::Absolute,
                right: px(20),
                bottom: px(20),
                flex_direction: FlexDirection::ColumnReverse,
                row_gap: px(8),
                align_items: AlignItems::FlexEnd,
            }
            GlobalZIndex(900)
            Pickable::IGNORE
            Children [ {item} ]
        });
    }
}

fn expire_toasts(
    time: Res<Time<Real>>,
    mut toasts: Query<(Entity, &mut ToastLife)>,
    mut commands: Commands,
) {
    for (entity, mut life) in &mut toasts {
        if life.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Dialogs
// ============================================================================

/// The root of a modal dialog.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Dialog;

/// On a dialog button: whether it confirms (`true`) or cancels (`false`).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DialogAnswer(pub bool);

/// Triggered on the [`Dialog`] entity when it is answered (a button, or Escape / gamepad
/// B for cancel), just before it despawns.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct DialogResult {
    pub entity: Entity,
    pub confirmed: bool,
}

/// A modal yes/no question over a dimmed backdrop. Cancel has focus first, so a stray
/// ENTER or A does not confirm something destructive.
///
/// ```ignore
/// commands
///     .spawn_scene(scenes::confirm("Quit?", "Unsaved progress is lost.", "Quit"))
///     .observe(|r: On<DialogResult>, mut exit: MessageWriter<AppExit>| {
///         if r.confirmed { exit.write(AppExit::Success); }
///     });
/// ```
pub fn confirm(
    title: impl Into<String>,
    message: impl Into<String>,
    confirm_label: impl Into<String>,
) -> impl Scene {
    let (title, message, confirm_label) = (title.into(), message.into(), confirm_label.into());
    bsn! {
        @backdrop()
        Dialog
        GlobalZIndex(800)
        Children [
            @card()
            Node { max_width: px(460) }
            Children [
                @header(title)
                --
                @caption(message)
                --
                @row(12.0)
                Node { justify_content: JustifyContent::FlexEnd, margin: {UiRect::top(px(8.0))} }
                Children [
                    @button("Cancel") Variant::Secondary DialogAnswer(false) AutoFocus
                    --
                    @button(confirm_label) Variant::Danger DialogAnswer(true)
                ]
            ]
        ]
    }
}

fn answer_dialog(
    activate: On<Activate>,
    answers: Query<&DialogAnswer>,
    parents: Query<&ChildOf>,
    dialogs: Query<(), With<Dialog>>,
    mut commands: Commands,
) {
    let Ok(answer) = answers.get(activate.entity) else {
        return;
    };
    if let Some(dialog) = parents
        .iter_ancestors(activate.entity)
        .find(|e| dialogs.contains(*e))
    {
        commands.trigger(DialogResult {
            entity: dialog,
            confirmed: answer.0,
        });
        commands.entity(dialog).despawn();
    }
}

/// Cancel the most recently spawned dialog. Returns whether there was one.
pub(crate) fn cancel_top_dialog(dialogs: &Query<Entity, With<Dialog>>, commands: &mut Commands) -> bool {
    let Some(dialog) = dialogs.iter().max() else {
        return false;
    };
    commands.trigger(DialogResult {
        entity: dialog,
        confirmed: false,
    });
    commands.entity(dialog).despawn();
    true
}
