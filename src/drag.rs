//! Drag and drop for UI nodes.
//!
//! Patch `@scenes::draggable()` onto a node and it follows the pointer while dragged
//! (lifted above its siblings, outlined in [`ColorToken::Accent`]), then snaps back to
//! its layout slot. Drop it on another node with `@scenes::drop_target()` and a [`Dropped`]
//! event fires on that target, naming what landed on it -- what the drop *means* (swap
//! two inventory slots, equip, sell) stays with the game:
//!
//! ```ignore
//! @slot() @scenes::draggable() @scenes::drop_target()
//! on(|drop: On<Dropped>, mut nodes: Query<&mut Node>| {
//!     if let Ok([mut a, mut b]) = nodes.get_many_mut([drop.entity, drop.dragged]) {
//!         core::mem::swap(&mut a.grid_column, &mut b.grid_column);
//!         core::mem::swap(&mut a.grid_row, &mut b.grid_row);
//!     }
//! })
//! ```
//!
//! The dragged node keeps its place in the layout (only its `UiTransform` moves), so
//! nothing reflows while dragging, and it can leave its container freely.

use bevy::picking::events::{PointerDrag, PointerDragDrop, PointerDragEnd, PointerDragStart};
use bevy::prelude::*;

use crate::{ColorToken, LavaTheme};

/// Drag-and-drop observers. Added by [`crate::LavaUiPlugin`].
pub struct DragPlugin;

impl Plugin for DragPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(start_drag)
            .add_observer(follow_drag)
            .add_observer(end_drag)
            .add_observer(drop_on_target);
    }
}

/// This node can be picked up and dragged.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(UiTransform, GlobalZIndex)]
pub struct Draggable;

/// Things can be dropped on this node; it receives [`Dropped`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct DropTarget;

/// Marks a node while it is being dragged.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Dragging;

/// Triggered on a [`DropTarget`] when a [`Draggable`] is released over it.
#[derive(EntityEvent, Clone, Copy, Debug)]
pub struct Dropped {
    /// The target.
    pub entity: Entity,
    /// What was dropped on it.
    pub dragged: Entity,
}

/// Make this node draggable. Its children should be `Pickable::IGNORE` (lava's widgets
/// already are) so the drag starts on the node itself.
///
/// The node lets pointer events through to what is under it while it moves
/// (`should_block_lower: false`), which is how a drop target underneath can see the drop.
pub fn draggable() -> impl Scene {
    bsn! {
        Draggable
        Pickable { should_block_lower: false, is_hoverable: true }
    }
}

/// Make this node accept drops: it receives [`Dropped`].
pub fn drop_target() -> impl Scene {
    bsn! { DropTarget }
}

fn start_drag(
    drag: On<PointerDragStart>,
    mut nodes: Query<&mut GlobalZIndex, With<Draggable>>,
    theme: Option<Res<LavaTheme>>,
    mut commands: Commands,
) {
    let Ok(mut z) = nodes.get_mut(drag.entity) else {
        return;
    };
    z.0 = 50;
    let color = theme.map_or(Color::WHITE, |t| ColorToken::Accent.color(&t));
    commands
        .entity(drag.entity)
        .insert((Dragging, Outline::new(px(3.0), Val::ZERO, color)));
}

fn follow_drag(drag: On<PointerDrag>, mut nodes: Query<&mut UiTransform, With<Dragging>>) {
    if let Ok(mut transform) = nodes.get_mut(drag.entity) {
        transform.translation = Val2::px(drag.distance.x, drag.distance.y);
    }
}

fn end_drag(
    drag: On<PointerDragEnd>,
    mut nodes: Query<(&mut UiTransform, &mut GlobalZIndex), With<Dragging>>,
    mut commands: Commands,
) {
    if let Ok((mut transform, mut z)) = nodes.get_mut(drag.entity) {
        transform.translation = Val2::ZERO;
        z.0 = 0;
        commands.entity(drag.entity).remove::<(Dragging, Outline)>();
    }
}

fn drop_on_target(
    drop: On<PointerDragDrop>,
    targets: Query<(), With<DropTarget>>,
    draggables: Query<(), With<Draggable>>,
    mut commands: Commands,
) {
    if drop.entity == drop.dropped
        || !targets.contains(drop.entity)
        || !draggables.contains(drop.dropped)
    {
        return;
    }
    commands.trigger(Dropped {
        entity: drop.entity,
        dragged: drop.dropped,
    });
}
