//! Screens: one Bevy `States` value per screen, spawned on enter and despawned on exit.
//!
//! ```ignore
//! #[derive(States, Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
//! enum Menu { #[default] Main, Audio }
//!
//! app.init_state::<Menu>()
//!     .add_screen(Menu::Main, main_menu)
//!     .add_screen(Menu::Audio, audio_menu)
//!     .screen_back(Menu::Audio, Menu::Main);  // Escape / gamepad B / a `Back` button
//!
//! fn main_menu() -> impl Scene {
//!     bsn! { @scenes::backdrop() Children [ @scenes::card() Children [
//!         @scenes::button("Audio") @scenes::go_to(Menu::Audio)
//!     ] ] }
//! }
//! fn audio_menu() -> impl Scene {
//!     bsn! { ... @scenes::button("Back") Back }
//! }
//! ```
//!
//! No marker components, no show/hide systems: the state *is* the navigation. A screen
//! that is re-entered is rebuilt from its scene, so anything it shows should be bound to
//! game state ([`crate::bind`]) rather than written once.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
use bevy::ui_widgets::Activate;

use crate::focus::NavigateBack;

/// On a button: activating it switches state `S` to this value. Add it with
/// [`go_to`], which spares you the turbofish BSN would otherwise need:
///
/// ```ignore
/// @scenes::button("Graphics") @scenes::go_to(Menu::Graphics)
/// ```
#[derive(Component)]
pub struct GoTo<S: States>(pub S);

impl<S: States> Clone for GoTo<S> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S: States + Default> Default for GoTo<S> {
    fn default() -> Self {
        Self(S::default())
    }
}

/// Make this button switch state `S` to `state` when activated.
pub fn go_to<S: FreelyMutableState + Default + Unpin>(state: S) -> impl Scene {
    let go_to = GoTo(state);
    bsn! { ~{go_to} }
}

/// On a button: activating it goes back, exactly like Escape / gamepad B -- to the parent
/// registered with [`ScreenAppExt::screen_back`], or closes whatever dialog or dropdown is
/// open first.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Back;

/// Where "back" leads from each screen of state type `S`.
#[derive(Resource)]
pub struct ScreenParents<S: States>(pub HashMap<S, S>);

impl<S: States> Default for ScreenParents<S> {
    fn default() -> Self {
        Self(HashMap::default())
    }
}

/// Registers screens on an [`App`].
pub trait ScreenAppExt {
    /// Spawn `scene()` when entering `state`, and despawn it when leaving. The state type
    /// must already be initialised (`app.init_state::<S>()`).
    fn add_screen<S, F, Sc>(&mut self, state: S, scene: F) -> &mut Self
    where
        S: FreelyMutableState + Default,
        F: Fn() -> Sc + Send + Sync + 'static,
        Sc: Scene;

    /// Going back from `from` (Escape, gamepad B, a [`Back`] button) leads to `to`.
    fn screen_back<S: FreelyMutableState + Default>(&mut self, from: S, to: S) -> &mut Self;
}

impl ScreenAppExt for App {
    fn add_screen<S, F, Sc>(&mut self, state: S, scene: F) -> &mut Self
    where
        S: FreelyMutableState + Default,
        F: Fn() -> Sc + Send + Sync + 'static,
        Sc: Scene,
    {
        register_state_observers::<S>(self);
        let scoped = state.clone();
        self.add_systems(OnEnter(state), move |mut commands: Commands| {
            commands
                .spawn_scene(scene())
                .insert(DespawnOnExit(scoped.clone()));
        })
    }

    fn screen_back<S: FreelyMutableState + Default>(&mut self, from: S, to: S) -> &mut Self {
        register_state_observers::<S>(self);
        self.world_mut()
            .get_resource_or_insert_with(ScreenParents::<S>::default)
            .0
            .insert(from, to);
        self
    }
}

/// The `GoTo<S>` and back observers, once per state type.
fn register_state_observers<S: FreelyMutableState + Default>(app: &mut App) {
    if app.world().contains_resource::<ScreenParents<S>>() {
        return;
    }
    app.init_resource::<ScreenParents<S>>()
        .add_observer(switch_on_go_to::<S>)
        .add_observer(go_back::<S>);
}

fn switch_on_go_to<S: FreelyMutableState + Default>(
    activate: On<Activate>,
    targets: Query<&GoTo<S>>,
    next: Option<ResMut<NextState<S>>>,
) {
    if let (Ok(target), Some(mut next)) = (targets.get(activate.entity), next) {
        next.set(target.0.clone());
    }
}

fn go_back<S: FreelyMutableState + Default>(
    _: On<NavigateBack>,
    parents: Res<ScreenParents<S>>,
    current: Option<Res<State<S>>>,
    next: Option<ResMut<NextState<S>>>,
) {
    let (Some(current), Some(mut next)) = (current, next) else {
        return;
    };
    if let Some(parent) = parents.0.get(current.get()) {
        next.set(parent.clone());
    }
}
