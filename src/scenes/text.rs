//! Text scenes. All but [`text`] are themed: font and colour come from tokens.
//!
//! To recolour a themed text, patch its *token*, not its `TextColor` (the token system
//! repaints a bare `TextColor` every frame):
//!
//! ```ignore
//! @scenes::label("Low ammo!") ThemedTextColor(ColorToken::Danger)
//! ```

use bevy::prelude::*;

use crate::{ColorToken, FontToken, ThemedFont, ThemedTextColor};

/// The one big line on a title screen.
pub fn title(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Title)
        ThemedTextColor(ColorToken::HeaderText)
    }
}

/// A header label: theme header font and color.
pub fn header(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Header)
        ThemedTextColor(ColorToken::HeaderText)
    }
}

/// A body label: theme label font and color.
pub fn label(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Label)
        ThemedTextColor(ColorToken::LabelText)
    }
}

/// Small, muted text: hints, captions, table headings, "press A to select".
pub fn caption(text: impl Into<String>) -> impl Scene {
    let text = text.into();
    bsn! {
        Text({text})
        ThemedFont(FontToken::Small)
        ThemedTextColor(ColorToken::TextMuted)
    }
}

/// A dim section-divider label. Same as [`caption`]; kept under its old name.
pub fn section_label(content: impl Into<String>) -> impl Scene {
    caption(content)
}

/// A text node with an explicit size and color, carrying **no** theme tokens.
///
/// The escape hatch for text whose colour is truly content, not style -- a player's chosen
/// team colour, say. For *status* colours (red for danger, green for success) prefer a
/// themed label with [`ColorToken::Danger`] / [`ColorToken::Success`], which follow a
/// theme swap. Because there is no token, a patched `TextColor` on this one sticks.
pub fn text(content: impl Into<String>, size: f32, color: Color) -> impl Scene {
    let content = content.into();
    bsn! {
        Text({content})
        TextFont { font_size: {px(size)} }
        TextColor({color})
    }
}
