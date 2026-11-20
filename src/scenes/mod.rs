//! BSN scene functions — the declarative half of the library, and the API to use.
//!
//! Each function returns an `impl Scene`, so they compose inside `bsn!` and can be
//! patched by the caller without the widget having to expose an option for every field:
//!
//! ```ignore
//! commands.spawn_scene(bsn! {
//!     @scenes::backdrop()
//!     Children [
//!         @scenes::card()
//!         Children [
//!             @scenes::header("Paused")
//!             --
//!             @scenes::button("Resume") on(resume)
//!             --
//!             // same widget, one thing changed -- this is patching
//!             @scenes::button("Quit") Variant::Danger on(quit)
//!         ]
//!     ]
//! });
//! ```
//!
//! Widgets take no theme argument. Colors and fonts are [`crate::tokens`] resolved
//! against the `LavaTheme` resource each frame, so a theme swap repaints live UI instead
//! of forcing a rebuild. To change a widget's colour, patch its *token* or *variant*:
//!
//! ```ignore
//! @scenes::label("Low ammo") ThemedTextColor(ColorToken::Danger)
//! @scenes::button("Delete") Variant::Danger
//! ```
//!
//! Sizes are layout, not theme: patch `Node` (`@scenes::button("Ok") Node { width: px(200) }`).
//!
//! Two rules when writing scenes:
//!
//! * `Scene` is `Send + Sync + 'static`, so a scene may not borrow anything. Copy values
//!   into locals *before* the `bsn!` block.
//! * anything that is not BSN value syntax (a literal, a path, a struct/tuple literal or
//!   a `#Name`) has to be wrapped in braces: `Text({text})`, not `Text(text.into())`. A
//!   whole component *value* goes in `~{value}`.

mod buttons;
mod layout;
mod meters;
mod text;

pub use buttons::*;
pub use layout::*;
pub use meters::*;
pub use text::*;

pub use crate::bind::{
    bind_checked, bind_choice, bind_input, bind_meter, bind_meter_to, bind_slider, bind_text,
    bind_text_to, bind_visible,
};
pub use crate::controls::{
    checkbox, choice, cycle, input_text, select, slider, text_area, text_input, toggle,
};
pub use crate::drag::{draggable, drop_target};
pub use crate::media::{icon, image, nine_slice, skinned_button};
pub use crate::overlay::{confirm, tooltip};
pub use crate::responsive::{full_width_below, stack_below};
pub use crate::screens::go_to;

use bevy::prelude::*;

/// Replace `parent`'s children with a freshly built [`SceneList`].
///
/// The scene-API answer to `UIBuilder::start_from_entity(.., clear_children: true)`: the
/// parent entity survives, so anything referring to it (a marker query, a stored
/// `Entity`, a `WorldFollower` target) keeps working, and only the contents are rebuilt.
///
/// ```ignore
/// scenes::replace_children(&mut commands, list_root, bsn_list! {
///     @scenes::list_item("one", false)
///     --
///     @scenes::list_item("two", true)
/// });
/// ```
///
/// The new children are spawned in the `SpawnScene` schedule, i.e. after `Update` in the
/// same frame -- so a system that rebuilds a list cannot also query the result in that
/// same run.
pub fn replace_children(commands: &mut Commands, parent: Entity, children: impl SceneList) {
    commands
        .entity(parent)
        .despawn_related::<Children>()
        .queue_spawn_related_scenes::<Children>(children);
}
