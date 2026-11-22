//! Images: plain, icon-sized, nine-slice, and nine-slice *skinned buttons* whose tint
//! follows the theme.
//!
//! A skin is a greyscale (ideally white) image, so the theme decides its colour: a
//! skinned button multiplies its image by the same [`crate::Variant`] colours a flat
//! button would paint as its background. One frame image serves every variant and every
//! theme.

use bevy::prelude::*;
use bevy::ui::widget::NodeImageMode;
use bevy::ui_widgets::Button as WidgetsButton;
use bevy::picking::hover::Hovered;

use crate::scenes::focusable;
use crate::{FontToken, ThemedFont, Variant, VariantText};

/// On an entity with an `ImageNode` and an `InteractionPalette`: the palette tints the
/// image instead of filling the background. Set by [`skinned_button`].
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct PaletteTintsImage;

/// An image from an asset path, at its natural size (patch `Node` to size it).
pub fn image(path: impl Into<String>) -> impl Scene {
    let path = path.into();
    bsn! { ImageNode { image: {path} } }
}

/// A square image `size` pixels wide: item icons, avatars, status icons.
pub fn icon(path: impl Into<String>, size: f32) -> impl Scene {
    let path = path.into();
    bsn! {
        ImageNode { image: {path} }
        Node { width: {px(size)}, height: {px(size)} }
    }
}

fn sliced(border: f32) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

/// A nine-slice image: the `border`-pixel corners keep their size and the edges and
/// middle stretch, so one small frame image fits a node of any size. Size it by patching
/// `Node`; put content in `Children [..]`.
pub fn nine_slice(path: impl Into<String>, border: f32) -> impl Scene {
    let path = path.into();
    let mode = sliced(border);
    bsn! {
        ImageNode { image: {path}, image_mode: {mode} }
        Node {
            padding: {UiRect::all(px(border))},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
    }
}

/// A button drawn with a nine-slice skin instead of a flat rounded rectangle. The skin is
/// tinted by the button's [`Variant`] (and its selected / disabled state), so it follows
/// the theme like any other button. Same events and focus behaviour as
/// [`crate::scenes::button`].
pub fn skinned_button(text: impl Into<String>, skin: impl Into<String>, border: f32) -> impl Scene {
    let text = text.into();
    let skin = skin.into();
    let mode = sliced(border);
    bsn! {
        ImageNode { image: {skin}, image_mode: {mode} }
        PaletteTintsImage
        Node {
            min_width: px(150),
            height: px(54),
            padding: {UiRect::horizontal(px(24.0 + border))},
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
        }
        WidgetsButton
        Hovered
        @focusable()
        Variant::Primary
        Children [
            Text({text})
            ThemedFont(FontToken::Button)
            VariantText
            Pickable::IGNORE
        ]
    }
}
