//! Progress bars, meters and collapsible sections.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button as WidgetsButton;

use crate::scenes::focusable;
use crate::{
    CollapseToggleButton, Collapsible, CollapsibleContent, ColorToken, FontToken, ProgressBar,
    ProgressBarFill, ThemedBackground, ThemedFont, ThemedPalette, ThemedTextColor,
};

/// A horizontal progress bar with explicit colours. Prefer [`meter`], which is themed.
pub fn progress_bar(value: f32, width: f32, height: f32, fill: Color, bg: Color) -> impl Scene {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    bsn! {
        ProgressBar { value: {value}, fill_color: {fill} }
        Node { width: {px(width)}, height: {px(height)}, overflow: {Overflow::clip()} }
        BackgroundColor({bg})
        Children [
            ProgressBarFill
            Node { width: {percent(pct)}, height: percent(100) }
            BackgroundColor({fill})
        ]
    }
}

/// A themed horizontal meter: the empty part is [`ColorToken::MeterBg`], the fill is the
/// `fill` token -- usually [`ColorToken::MeterFill`] or [`ColorToken::HealthFill`].
///
/// Drive it by writing [`ProgressBar::value`], or bind it to game state with
/// [`crate::bind::bind_meter`] / [`crate::bind::bind_meter_to`] and write nothing.
pub fn meter(value: f32, width: f32, height: f32, fill: ColorToken) -> impl Scene {
    let pct = value.clamp(0.0, 1.0) * 100.0;
    bsn! {
        ProgressBar { value: {value} }
        Node {
            width: {px(width)},
            height: {px(height)},
            overflow: {Overflow::clip()},
            border_radius: {BorderRadius::all(px(height * 0.3))},
        }
        ThemedBackground(ColorToken::MeterBg)
        Children [
            ProgressBarFill
            Node { width: {percent(pct)}, height: percent(100) }
            ThemedBackground({fill})
        ]
    }
}

/// A collapsible section: a toggle button plus a content container that hides when
/// collapsed.
///
/// The `#Section` name ties the three entities together. Both `CollapseToggleButton` and
/// `CollapsibleContent` hold an `Entity`, and a `#Name` only resolves inside the `bsn!`
/// invocation that declares it -- which is why the whole widget is one scene rather than
/// three the caller assembles.
pub fn collapsible(
    label_text: impl Into<String>,
    collapsed: bool,
    content: impl SceneList,
) -> impl Scene {
    let label_text = label_text.into();
    let arrow = if collapsed { "\u{25b6}" } else { "\u{25bc}" };
    let toggle_text = format!("{arrow} {label_text}");
    let display = if collapsed {
        Display::None
    } else {
        Display::Flex
    };
    bsn! {
        #Section
        Collapsible { collapsed: {collapsed}, label: {label_text} }
        Node { flex_direction: FlexDirection::Column }
        Children [
                CollapseToggleButton { target: #Section }
                WidgetsButton
                Hovered
                @focusable()
                ThemedPalette {
                    none: ColorToken::CollapsibleBg,
                    hovered: ColorToken::CollapsibleBgHovered,
                    pressed: ColorToken::CollapsibleBgPressed,
                }
                Node {
                    padding: {UiRect::all(px(8.0))},
                    margin: {UiRect::bottom(px(4.0))},
                }
                Children [
                    Text({toggle_text})
                    ThemedFont(FontToken::Label)
                    ThemedTextColor(ColorToken::ButtonText)
                    Pickable::IGNORE
                ]
            --
                CollapsibleContent { parent: #Section }
                Node {
                    flex_direction: FlexDirection::Column,
                    width: percent(100),
                    display: {display},
                }
                Children [ {content} ]
        ]
    }
}
