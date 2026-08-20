use crate::{
    ButtonBuilder, ButtonVariant, CollapseToggleButton, Collapsible, CollapsibleContent,
    CollapsibleTogglePosition, InteractionPalette, LavaTheme, TextStyle, UIBuilder,
};
use bevy::ecs::event::EntityEvent;
use bevy::ecs::system::IntoObserverSystem;
use bevy::feathers::cursor::EntityCursor;
use bevy::feathers::font_styles::InheritableFont;
use bevy::feathers::handle_or_path::HandleOrPath;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{Justify, LineBreak, TextLayout};
use bevy::ui_widgets::{Activate, Button as WidgetsButton};
use bevy::window::SystemCursorIcon;
use std::collections::VecDeque;

impl<'w, 's> UIBuilder<'w, 's> {
    /// Get a reference to the UI theme.
    pub fn theme(&self) -> &LavaTheme {
        &self.theme
    }

    /// Create a new root UI container with a default `Node`.
    pub fn new(mut commands: Commands<'w, 's>, theme: Option<LavaTheme>) -> Self {
        let entity = commands.spawn(Node::default()).id();
        Self {
            commands,
            theme: theme.unwrap_or_default(),
            root_entity: entity,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }

    /// Start building from an existing entity (useful for rebuilding UI at runtime).
    pub fn start_from_entity(
        mut commands: Commands<'w, 's>,
        entity: Entity,
        clear_children: bool,
        theme: Option<LavaTheme>,
    ) -> Self {
        if clear_children {
            commands.entity(entity).despawn_related::<Children>();
        }
        Self {
            commands,
            theme: theme.unwrap_or_default(),
            root_entity: entity,
            current_entity: entity,
            parent_stack: VecDeque::new(),
        }
    }

    /// Get the entity currently being built.
    pub fn current_entity(&self) -> Entity {
        self.current_entity
    }

    // ========================================================================
    // Tree navigation
    // ========================================================================

    /// Spawn a new child `Node` and push the current entity onto the parent stack.
    pub fn child(&mut self) -> &mut Self {
        self.parent_stack.push_back(self.current_entity);
        let child = self.commands.spawn(Node::default()).id();
        self.commands.entity(self.current_entity).add_child(child);
        self.current_entity = child;
        self
    }

    /// Pop back to the parent entity.
    pub fn parent(&mut self) -> &mut Self {
        if let Some(parent) = self.parent_stack.pop_back() {
            self.current_entity = parent;
        }
        self
    }

    /// Spawn a child, run a closure on it, then return to the parent.
    pub fn with_child<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();
        self.child();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self)));

        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
        self
    }

    /// Iterate data and spawn a child for each item.
    pub fn foreach_child<I, F>(&mut self, iter: I, mut f: F) -> &mut Self
    where
        I: IntoIterator,
        F: FnMut(&mut Self, I::Item),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        for item in iter {
            self.child();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(self, item)));
            self.current_entity = original_entity;
            while self.parent_stack.len() > original_stack_len {
                self.parent_stack.pop_back();
            }
            if let Err(payload) = result {
                std::panic::resume_unwind(payload);
            }
        }
        self
    }

    /// Finish building and return the root entity + commands.
    pub fn build(self) -> (Entity, Commands<'w, 's>) {
        (self.root_entity, self.commands)
    }

    // ========================================================================
    // Component insertion
    // ========================================================================

    /// Attach an entity-scoped observer to the current entity.
    pub fn observe<E: EntityEvent, B: Bundle, M>(
        &mut self,
        handler: impl IntoObserverSystem<E, B, M>,
    ) -> &mut Self {
        self.commands.entity(self.current_entity).observe(handler);
        self
    }

    /// Insert any component on the current entity.
    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.commands.entity(self.current_entity).insert(component);
        self
    }

    /// Insert a bundle on the current entity.
    pub fn insert_bundle(&mut self, bundle: impl Bundle) -> &mut Self {
        self.commands.entity(self.current_entity).insert(bundle);
        self
    }

    /// Insert a default component on the current entity.
    pub fn component<T: Component + Default>(&mut self) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<T>()
            .or_default();
        self
    }

    // ========================================================================
    // Node — direct replacement (preferred for static layouts)
    // ========================================================================

    /// Replace the entire `Node` on the current entity.
    pub fn set_node(&mut self, node: Node) -> &mut Self {
        self.commands.entity(self.current_entity).insert(node);
        self
    }

    /// Mutate the existing `Node` on the current entity via a closure.
    pub fn modify_node(&mut self, f: impl FnOnce(Mut<Node>) + Send + Sync + 'static) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<Node>()
            .and_modify(f);
        self
    }

    // ========================================================================
    // Convenience layout setters (kept slim — only the most-used ones)
    // ========================================================================

    pub fn absolute_position(&mut self) -> &mut Self {
        self.modify_node(move |mut n| n.position_type = PositionType::Absolute)
    }

    pub fn bottom(&mut self, bottom: Val) -> &mut Self {
        self.modify_node(move |mut n| n.bottom = bottom)
    }
    pub fn left(&mut self, left: Val) -> &mut Self {
        self.modify_node(move |mut n| n.left = left)
    }
    pub fn top(&mut self, top: Val) -> &mut Self {
        self.modify_node(move |mut n| n.top = top)
    }
    pub fn right(&mut self, right: Val) -> &mut Self {
        self.modify_node(move |mut n| n.right = right)
    }
    pub fn display(&mut self, display: Display) -> &mut Self {
        self.modify_node(move |mut n| n.display = display)
    }
    pub fn display_flex(&mut self) -> &mut Self {
        self.display(Display::Flex)
    }
    pub fn display_grid(&mut self) -> &mut Self {
        self.display(Display::Grid)
    }
    pub fn display_none(&mut self) -> &mut Self {
        self.display(Display::None)
    }

    pub fn width(&mut self, width: Val) -> &mut Self {
        self.modify_node(move |mut n| n.width = width)
    }
    pub fn width_px(&mut self, w: f32) -> &mut Self {
        self.width(Val::Px(w))
    }
    pub fn width_percent(&mut self, w: f32) -> &mut Self {
        self.width(Val::Percent(w))
    }
    pub fn width_auto(&mut self) -> &mut Self {
        self.width(Val::Auto)
    }

    pub fn height(&mut self, height: Val) -> &mut Self {
        self.modify_node(move |mut n| n.height = height)
    }
    pub fn height_px(&mut self, h: f32) -> &mut Self {
        self.height(Val::Px(h))
    }
    pub fn height_percent(&mut self, h: f32) -> &mut Self {
        self.height(Val::Percent(h))
    }

    pub fn size_px(&mut self, w: f32, h: f32) -> &mut Self {
        self.width_px(w).height_px(h)
    }
    pub fn size_percent(&mut self, w: f32, h: f32) -> &mut Self {
        self.width_percent(w).height_percent(h)
    }
    pub fn size(&mut self, w: Val, h: Val) -> &mut Self {
        self.width(w).height(h)
    }

    pub fn size_scaled(&mut self, w_percent: f32, h_percent: f32) -> &mut Self {
        let width = w_percent * self.theme.ui_width / 100.0;
        let height = h_percent * self.theme.ui_height / 100.0;
        self.width(px(width)).height(px(height))
    }

    pub fn width_scaled(&mut self, w_percent: f32) -> &mut Self {
        let width = w_percent * self.theme.ui_width / 100.0;
        self.width(px(width))
    }

    pub fn height_scaled(&mut self, h_percent: f32) -> &mut Self {
        let height = h_percent * self.theme.ui_height / 100.0;
        self.height(px(height))
    }

    pub fn flex_direction(&mut self, dir: FlexDirection) -> &mut Self {
        self.modify_node(move |mut n| n.flex_direction = dir)
    }
    pub fn flex_row(&mut self) -> &mut Self {
        self.flex_direction(FlexDirection::Row)
    }
    pub fn flex_column(&mut self) -> &mut Self {
        self.flex_direction(FlexDirection::Column)
    }

    pub fn align_items(&mut self, align: AlignItems) -> &mut Self {
        self.modify_node(move |mut n| n.align_items = align)
    }
    pub fn align_items_center(&mut self) -> &mut Self {
        self.align_items(AlignItems::Center)
    }
    pub fn align_items_end(&mut self) -> &mut Self {
        self.align_items(AlignItems::End)
    }
    pub fn align_items_start(&mut self) -> &mut Self {
        self.align_items(AlignItems::Start)
    }

    pub fn justify_content(&mut self, jc: JustifyContent) -> &mut Self {
        self.modify_node(move |mut n| n.justify_content = jc)
    }
    pub fn justify_center(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::Center)
    }

    pub fn padding(&mut self, p: UiRect) -> &mut Self {
        self.modify_node(move |mut n| n.padding = p)
    }
    pub fn padding_all_px(&mut self, v: f32) -> &mut Self {
        self.padding(UiRect::all(Val::Px(v)))
    }
    pub fn padding_all(&mut self, v: Val) -> &mut Self {
        self.padding(UiRect::all(v))
    }

    pub fn padding_top(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.padding.top = v)
    }
    pub fn padding_left(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.padding.left = v)
    }

    pub fn padding_right(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.padding.right = v)
    }

    pub fn padding_bottom(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.padding.bottom = v)
    }

    pub fn margin(&mut self, m: UiRect) -> &mut Self {
        self.modify_node(move |mut n| n.margin = m)
    }
    pub fn margin_all_px(&mut self, v: f32) -> &mut Self {
        self.margin(UiRect::all(Val::Px(v)))
    }
    pub fn margin_btm_px(&mut self, v: f32) -> &mut Self {
        self.margin(UiRect::bottom(Val::Px(v)))
    }
    pub fn margin_btm(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.margin.bottom = v)
    }
    pub fn margin_top(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.margin.top = v)
    }
    pub fn margin_left(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.margin.left = v)
    }
    pub fn margin_right(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.margin.right = v)
    }
    pub fn margin_all(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.margin = UiRect::all(v))
    }
    pub fn margin_zero(&mut self) -> &mut Self {
        self.margin(UiRect::ZERO)
    }
    pub fn padding_zero(&mut self) -> &mut Self {
        self.padding(UiRect::ZERO)
    }

    pub fn row_gap_px(&mut self, gap: f32) -> &mut Self {
        self.modify_node(move |mut n| n.row_gap = Val::Px(gap))
    }

    pub fn column_gap_px(&mut self, gap: f32) -> &mut Self {
        self.modify_node(move |mut n| n.column_gap = Val::Px(gap))
    }

    pub fn column_gap(&mut self, gap: Val) -> &mut Self {
        self.modify_node(move |mut n| n.column_gap = gap)
    }

    pub fn row_gap(&mut self, gap: Val) -> &mut Self {
        self.modify_node(move |mut n| n.row_gap = gap)
    }

    pub fn gap(&mut self, col_gap: Val, row_gap: Val) -> &mut Self {
        self.modify_node(move |mut n| {
            n.row_gap = row_gap;
            n.column_gap = col_gap;
        })
    }
    pub fn gap_px(&mut self, gap: f32) -> &mut Self {
        self.modify_node(move |mut n| {
            n.row_gap = Val::Px(gap);
            n.column_gap = Val::Px(gap);
        })
    }

    pub fn grid_cols(&mut self, count: u16) -> &mut Self {
        self.modify_node(move |mut n| {
            n.grid_template_columns = vec![RepeatedGridTrack::fr(count, 1.0)];
        })
    }

    pub fn border_radius_all_px(&mut self, v: f32) -> &mut Self {
        self.modify_node(move |mut n| n.border_radius = BorderRadius::all(Val::Px(v)))
    }

    pub fn border_all_px(&mut self, width: f32, color: Color) -> &mut Self {
        self.border(UiRect::all(Val::Px(width)), color)
    }

    pub fn border_all(&mut self, width: Val, color: Color) -> &mut Self {
        self.border(UiRect::all(width), color)
    }

    pub fn flex_wrap(&mut self) -> &mut Self {
        self.modify_node(move |mut n| n.flex_wrap = FlexWrap::Wrap)
    }

    pub fn with_overflow(&mut self, overflow: Overflow) -> &mut Self {
        self.modify_node(move |mut n| n.overflow = overflow)
    }
    pub fn overflow_scroll_y(&mut self) -> &mut Self {
        self.with_overflow(Overflow::scroll_y())
    }

    pub fn justify_start(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::FlexStart)
    }
    pub fn justify_space_between(&mut self) -> &mut Self {
        self.justify_content(JustifyContent::SpaceBetween)
    }

    pub fn with_flex_shrink(&mut self, shrink: f32) -> &mut Self {
        self.modify_node(move |mut n| n.flex_shrink = shrink)
    }

    pub fn with_flex_grow(&mut self, grow: f32) -> &mut Self {
        self.modify_node(move |mut n| n.flex_grow = grow)
    }

    // ========================================================================
    // Colors
    // ========================================================================

    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<BackgroundColor>()
            .and_modify(move |mut bg| *bg = BackgroundColor(color))
            .or_insert(BackgroundColor(color));
        self
    }

    pub fn border(&mut self, border: UiRect, color: Color) -> &mut Self {
        self.modify_node(move |mut n| n.border = border);
        self.commands
            .entity(self.current_entity)
            .entry::<BorderColor>()
            .and_modify(move |mut bc| *bc = BorderColor::all(color))
            .or_insert(BorderColor::all(color));
        self
    }

    // ========================================================================
    // Text
    // ========================================================================

    /// Add a text component to the current entity, with optional style overrides.
    /// `None` for `style` uses all theme defaults.
    pub fn with_text(&mut self, text: impl Into<String>, style: Option<TextStyle>) -> &mut Self {
        let theme = &self.theme.text;
        let s = style.unwrap_or_default();
        let bundle = (
            Text::new(text.into()),
            TextFont::default()
                .with_font(s.font.unwrap_or_else(|| theme.font.clone()))
                .with_font_size(s.font_size.unwrap_or(theme.label_size)),
            TextColor(s.color.unwrap_or(theme.label_color)),
            TextLayout::new(
                s.justify.unwrap_or_default(),
                s.line_break.unwrap_or_default(),
            ),
        );
        self.commands.entity(self.current_entity).insert(bundle);
        self
    }

    /// Shorthand: set text with all theme defaults.
    pub fn default_text(&mut self, text: impl Into<String>) -> &mut Self {
        self.with_text(text, None)
    }

    /// Spawn a child with text and optional style overrides.
    pub fn add_text_child(
        &mut self,
        text: impl Into<String>,
        style: Option<TextStyle>,
    ) -> &mut Self {
        self.with_child(|b| {
            b.with_text(text, style);
        })
    }

    /// Spawn a child with text using all theme defaults.
    pub fn text_node(&mut self, text: impl Into<String>) -> &mut Self {
        self.add_text_child(text, None)
    }

    pub fn text_justify(&mut self, justify: Justify) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .entry::<TextLayout>()
            .and_modify(move |mut tl| tl.justify = justify)
            .or_insert(TextLayout::new(justify, LineBreak::default()));
        self
    }
    pub fn text_justify_center(&mut self) -> &mut Self {
        self.text_justify(Justify::Center)
    }

    // ========================================================================
    // Composite helpers (row, column, grid, panel, etc.)
    // ========================================================================

    pub fn add_row<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        self.with_child(|ui| {
            ui.display_flex().flex_row();
            f(ui);
        })
    }

    pub fn add_column<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        self.with_child(|ui| {
            ui.display_flex().flex_column();
            f(ui);
        })
    }

    pub fn add_panel<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        self.with_child(|ui| {
            ui.display_flex().flex_column().padding_all_px(16.0);
            f(ui);
        })
    }

    pub fn add_grid<F: FnOnce(&mut Self)>(&mut self, cols: u16, f: F) -> &mut Self {
        self.with_child(|ui| {
            ui.display_grid().grid_cols(cols).gap_px(8.0);
            f(ui);
        })
    }

    pub fn add_centered<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        self.with_child(|ui| {
            ui.display_flex().justify_center().align_items_center();
            f(ui);
        })
    }

    /// Add a simple button child with text, size, bg color, font size, border radius, and a marker component.
    pub fn add_button<T: Component>(
        &mut self,
        text: impl Into<String>,
        width: f32,
        height: f32,
        bg_color: Color,
        font_size: f32,
        border_radius: f32,
        component: T,
    ) -> &mut Self {
        self.with_child(|ui| {
            ui.insert(Button)
                .insert(component)
                .width_px(width)
                .height_px(height)
                .bg_color(bg_color)
                .border_radius_all_px(border_radius)
                .display_flex()
                .justify_center()
                .align_items_center();
            ui.add_text_child(text, Some(crate::TextStyle::size(font_size)));
        })
    }

    // ========================================================================
    // Themed text shorthands — use the builder's own theme, no &t needed
    // ========================================================================

    /// Spawn a child with a header label using the builder's theme.
    pub fn themed_header(&mut self, text: impl Into<String>) -> &mut Self {
        let t = self.theme.text.clone();
        self.with_child(|c| {
            c.with_text(
                text,
                Some(crate::TextStyle::size_color(t.header_size, t.header_color)),
            );
        })
    }

    /// Spawn a child with a label using the builder's theme defaults.
    pub fn themed_label(&mut self, text: impl Into<String>) -> &mut Self {
        let t = self.theme.text.clone();
        self.with_child(|c| {
            c.with_text(
                text,
                Some(crate::TextStyle::size_color(t.label_size, t.label_color)),
            );
        })
    }

    /// Spawn a child label with explicit size and color (no theme needed).
    /// Returns the **parent** — use `with_child` + `with_text` if you need to insert
    /// a marker component on the text node itself.
    pub fn label(&mut self, text: impl Into<String>, font_size: f32, color: Color) -> &mut Self {
        self.with_child(|c| {
            c.with_text(text, Some(crate::TextStyle::size_color(font_size, color)));
        })
    }

    /// Spawn a label child and run a closure on it before returning to the parent.
    /// Use this when you need to insert a marker on the text entity, e.g.:
    /// `ui.label_child("Aliens: 0", 14.0, color, |c| { c.insert(HudAlienCount); });`
    pub fn label_child<F: FnOnce(&mut Self)>(
        &mut self,
        text: impl Into<String>,
        font_size: f32,
        color: Color,
        f: F,
    ) -> &mut Self {
        self.with_child(|c| {
            c.with_text(text, Some(crate::TextStyle::size_color(font_size, color)));
            f(c);
        })
    }

    /// Spawn a dim section-divider label ("-- Title --" style).
    pub fn section_label(&mut self, text: impl Into<String>) -> &mut Self {
        let color = Color::srgb(0.5, 0.8, 0.6);
        self.with_child(|c| {
            c.with_text(text, Some(crate::TextStyle::size_color(13.0, color)));
        })
    }

    // ========================================================================
    // Missing layout setters
    // ========================================================================

    pub fn min_width(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.min_width = v)
    }
    pub fn min_width_px(&mut self, v: f32) -> &mut Self {
        self.min_width(Val::Px(v))
    }
    pub fn max_width(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.max_width = v)
    }
    pub fn max_width_px(&mut self, v: f32) -> &mut Self {
        self.max_width(Val::Px(v))
    }
    pub fn min_height(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.min_height = v)
    }
    pub fn min_height_px(&mut self, v: f32) -> &mut Self {
        self.min_height(Val::Px(v))
    }
    pub fn max_height(&mut self, v: Val) -> &mut Self {
        self.modify_node(move |mut n| n.max_height = v)
    }
    pub fn max_height_px(&mut self, v: f32) -> &mut Self {
        self.max_height(Val::Px(v))
    }

    /// Add a flex_grow(1) spacer child to push siblings apart.
    pub fn spacer(&mut self) -> &mut Self {
        self.with_child(|c| {
            c.with_flex_grow(1.0);
        })
    }

    // ========================================================================
    // Structural composites
    // ========================================================================

    /// Full-height fixed-width column panel with padding, gap, and background.
    /// Matches the sidebar pattern used on every tool screen.
    pub fn side_panel<F: FnOnce(&mut Self)>(
        &mut self,
        width_px: f32,
        bg: Color,
        f: F,
    ) -> &mut Self {
        self.with_child(|c| {
            c.width_px(width_px)
                .height_percent(100.0)
                .flex_column()
                .padding_all_px(6.0)
                .row_gap_px(3.0)
                .bg_color(bg);
            f(c);
        })
    }

    /// Full-width scrollable flex-column list that grows to fill available space.
    pub fn scrollable_list<F: FnOnce(&mut Self)>(&mut self, f: F) -> &mut Self {
        self.with_child(|c| {
            c.display_flex()
                .flex_column()
                .gap_px(2.0)
                .with_flex_grow(1.0)
                .width_percent(100.0)
                .overflow_scroll_y();
            f(c);
        })
    }

    /// Like `scrollable_list` but capped at `max_height_px`.
    pub fn scrollable_list_bounded<F: FnOnce(&mut Self)>(
        &mut self,
        max_height_px: f32,
        f: F,
    ) -> &mut Self {
        self.with_child(|c| {
            c.display_flex()
                .flex_column()
                .gap_px(2.0)
                .width_percent(100.0)
                .max_height_px(max_height_px)
                .overflow_scroll_y();
            f(c);
        })
    }

    /// Selectable list item row. Highlights when `selected`.
    /// Uses the game-standard green palette for selected/hovered/unselected states.
    pub fn list_item<M>(
        &mut self,
        name: impl Into<String>,
        selected: bool,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        let bg = if selected {
            Color::srgba(0.15, 0.40, 0.20, 0.95)
        } else {
            Color::srgba(0.07, 0.12, 0.09, 0.85)
        };
        let tc = if selected {
            Color::srgb(0.9, 1.0, 0.9)
        } else {
            Color::srgb(0.65, 0.80, 0.65)
        };
        let name = name.into();
        self.with_child(|c| {
            c.width_percent(100.0)
                .padding(UiRect::axes(Val::Px(6.0), Val::Px(4.0)))
                .border_radius_all_px(3.0)
                .bg_color(bg)
                .insert(InteractionPalette {
                    none: bg,
                    hovered: Color::srgba(0.20, 0.50, 0.28, 0.95),
                    pressed: Color::srgba(0.12, 0.32, 0.18, 1.0),
                })
                .insert(bevy::picking::hover::Hovered::default())
                .insert(bevy::ui_widgets::Button)
                .observe(handler);
            c.with_child(|t| {
                t.with_text(name, Some(crate::TextStyle::size_color(11.0, tc)));
            });
        })
    }

    /// Small square button with a single glyph character.
    pub fn icon_button<M>(
        &mut self,
        glyph: impl Into<String>,
        size_px: f32,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        let glyph = glyph.into();
        self.with_child(|c| {
            c.size_px(size_px, size_px)
                .display_flex()
                .justify_center()
                .align_items_center()
                .border_radius_all_px(3.0)
                .bg_color(Color::srgba(0.25, 0.25, 0.30, 0.85))
                .insert(InteractionPalette {
                    none: Color::srgba(0.25, 0.25, 0.30, 0.85),
                    hovered: Color::srgba(0.40, 0.40, 0.50, 0.95),
                    pressed: Color::srgba(0.20, 0.20, 0.25, 1.0),
                })
                .insert(bevy::picking::hover::Hovered::default())
                .insert(bevy::ui_widgets::Button)
                .observe(handler);
            c.with_child(|t| {
                t.with_text(
                    glyph,
                    Some(crate::TextStyle::size_color(
                        12.0,
                        Color::srgb(0.9, 0.9, 0.9),
                    )),
                );
            });
        })
    }

    /// A 16×16 red delete button ("x"). Standard for list-row removal.
    pub fn delete_button<M>(
        &mut self,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.with_child(|c| {
            c.size_px(16.0, 16.0)
                .display_flex()
                .justify_center()
                .align_items_center()
                .bg_color(Color::srgba(0.4, 0.1, 0.1, 0.8))
                .insert(InteractionPalette {
                    none: Color::srgba(0.4, 0.1, 0.1, 0.8),
                    hovered: Color::srgba(0.6, 0.15, 0.15, 0.9),
                    pressed: Color::srgba(0.3, 0.08, 0.08, 1.0),
                })
                .insert(bevy::picking::hover::Hovered::default())
                .insert(bevy::ui_widgets::Button)
                .observe(handler);
            c.with_child(|t| {
                t.with_text(
                    "x",
                    Some(crate::TextStyle::size_color(
                        10.0,
                        Color::srgb(1.0, 0.7, 0.7),
                    )),
                );
            });
        })
    }

    /// Spawn a child with text and a fixed width.
    pub fn text_with_width(&mut self, text: impl Into<String>, width: f32) -> &mut Self {
        self.with_child(|ui| {
            ui.width_px(width).default_text(text);
        })
    }

    /// Spawn a child with text, run a closure on it, then return to parent.
    pub fn build_text_child<F: FnOnce(&mut Self)>(
        &mut self,
        text: impl Into<String>,
        f: F,
    ) -> &mut Self {
        self.with_child(|ui| {
            ui.default_text(text);
            f(ui);
        })
    }

    // ========================================================================
    // Button (imperative, using theme)
    // ========================================================================

    // ========================================================================
    // Private button helper
    // ========================================================================

    /// Shared implementation for all "spawn a themed button child" methods.
    /// Spawns a child entity with `bundle` (which must include `Node`, colors, etc.),
    /// attaches a text child, calls `setup(commands, button_entity)` for optional
    /// observer/extra-component attachment, then runs the `ButtonBuilder` closure.
    fn spawn_button_inner<B, S, F>(
        &mut self,
        text: impl Into<String>,
        bundle: B,
        setup: S,
        f: F,
    ) -> &mut Self
    where
        B: Bundle,
        S: FnOnce(&mut Commands, Entity),
        F: FnOnce(&mut ButtonBuilder),
    {
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        self.child();
        let button_entity = self.current_entity;

        self.commands.entity(button_entity).insert(bundle);
        setup(&mut self.commands, button_entity);

        let btn = &self.theme.button;
        let text_entity = self
            .commands
            .spawn((
                Text::new(text.into()),
                TextFont::default()
                    .with_font(btn.font.clone())
                    .with_font_size(btn.font_size),
                TextColor(btn.text_color),
            ))
            .id();
        self.commands.entity(button_entity).add_child(text_entity);

        let mut button_builder = ButtonBuilder {
            ui: self,
            text_entity: Some(text_entity),
        };
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut button_builder)));

        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
        self
    }

    // ========================================================================
    // Public button methods
    // ========================================================================

    /// Add a themed button as a child. Returns to the parent after the closure.
    pub fn add_themed_button<T, F>(&mut self, component: T, f: F) -> &mut Self
    where
        T: Component,
        F: FnOnce(&mut ButtonBuilder),
    {
        let btn = &self.theme.button;
        let bundle = (
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: btn.border_radius,
                ..default()
            },
            BackgroundColor(btn.bg),
            InteractionPalette {
                none: btn.bg,
                hovered: btn.bg_hovered,
                pressed: btn.bg_pressed,
            },
            WidgetsButton,
            component,
        );
        self.spawn_button_inner("", bundle, |_, _| {}, f)
    }

    pub fn z_index(&mut self, z_index: i32) -> &mut Self {
        self.commands
            .entity(self.current_entity)
            .insert(ZIndex(z_index));
        self
    }

    pub fn add_button_observe<M, F>(
        &mut self,
        text: impl Into<String>,
        f: F,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        F: FnOnce(&mut ButtonBuilder),
    {
        let btn = &self.theme.button;
        let bundle = (
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: btn.border_radius,
                border: UiRect::all(btn.border_width),
                ..default()
            },
            BorderColor::all(btn.border_color),
            BackgroundColor(btn.bg),
            InteractionPalette {
                hovered: btn.bg_hovered,
                pressed: btn.bg_pressed,
                none: btn.bg,
            },
            Hovered::default(),
            WidgetsButton,
            ButtonVariant::Normal,
            EntityCursor::System(SystemCursorIcon::Pointer),
            TabIndex(0),
        );
        self.spawn_button_inner(
            text,
            bundle,
            move |commands, entity| {
                commands.entity(entity).observe(handler);
            },
            f,
        )
    }

    /// Add a themed button (with width/height from theme) as a child. Returns to the parent after the closure.
    pub fn add_themed_button_observe<M, F>(
        &mut self,
        text: impl Into<String>,
        f: F,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        F: FnOnce(&mut ButtonBuilder),
    {
        let btn = &self.theme.button;
        let bundle = (
            Node {
                width: btn.width,
                height: btn.height,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: btn.border_radius,
                border: UiRect::all(btn.border_width),
                ..default()
            },
            BorderColor::all(btn.border_color),
            BackgroundColor(btn.bg),
            InteractionPalette {
                none: btn.bg,
                hovered: btn.bg_hovered,
                pressed: btn.bg_pressed,
            },
            Hovered::default(),
            WidgetsButton,
            ButtonVariant::Normal,
            EntityCursor::System(SystemCursorIcon::Pointer),
            TabIndex(0),
            InheritableFont {
                font: HandleOrPath::Handle(btn.font.clone()),
                font_size: btn.font_size,
            },
        );
        self.spawn_button_inner(
            text,
            bundle,
            move |commands, entity| {
                commands.entity(entity).observe(handler);
            },
            f,
        )
    }

    // ========================================================================
    // Collapsible sections
    // ========================================================================

    pub fn with_collapsible<F: FnOnce(&mut Self)>(
        &mut self,
        label: &str,
        initially_collapsed: bool,
        f: F,
    ) -> &mut Self {
        self.with_collapsible_toggle_at(
            label,
            initially_collapsed,
            CollapsibleTogglePosition::Top,
            f,
        )
    }

    /// Same as [`with_collapsible`](Self::with_collapsible), but the toggle button
    /// renders after the content instead of before it -- for panels anchored to the
    /// top of the screen, putting the toggle at the panel's bottom edge (closest to
    /// the rest of the UI/board) reads better than one pinned to the very top.
    pub fn with_collapsible_toggle_at_bottom<F: FnOnce(&mut Self)>(
        &mut self,
        label: &str,
        initially_collapsed: bool,
        f: F,
    ) -> &mut Self {
        self.with_collapsible_toggle_at(
            label,
            initially_collapsed,
            CollapsibleTogglePosition::Bottom,
            f,
        )
    }

    fn with_collapsible_toggle_at<F: FnOnce(&mut Self)>(
        &mut self,
        label: &str,
        initially_collapsed: bool,
        toggle_position: CollapsibleTogglePosition,
        f: F,
    ) -> &mut Self {
        let label_owned = label.to_string();
        let collapsed = initially_collapsed;

        self.with_child(|ui| {
            let collapsible_entity = ui.current_entity;
            ui.insert(if collapsed {
                Collapsible::collapsed(&label_owned)
            } else {
                Collapsible::new(&label_owned)
            });
            ui.display_flex().flex_column();

            let toggle_bg = ui.theme.button.collapsible_bg;
            let spawn_toggle = |ui: &mut Self| {
                ui.with_child(|btn| {
                    btn.insert(Button);
                    btn.insert(CollapseToggleButton {
                        target: collapsible_entity,
                    });
                    btn.padding_all_px(8.0)
                        .margin(UiRect::bottom(Val::Px(4.0)))
                        .bg_color(toggle_bg);

                    let arrow = if collapsed { "▶" } else { "▼" };
                    btn.add_text_child(
                        format!("{arrow} {label_owned}"),
                        Some(crate::TextStyle::size(12.0)),
                    );
                });
            };
            let spawn_content = |ui: &mut Self| {
                ui.with_child(|content| {
                    content.insert(CollapsibleContent {
                        parent: collapsible_entity,
                    });
                    content.display_flex().flex_column().width_percent(100.0);
                    if collapsed {
                        content.display(Display::None);
                    }
                    f(content);
                });
            };

            match toggle_position {
                CollapsibleTogglePosition::Top => {
                    spawn_toggle(ui);
                    spawn_content(ui);
                }
                CollapsibleTogglePosition::Bottom => {
                    spawn_content(ui);
                    spawn_toggle(ui);
                }
            }
        })
    }

    // ========================================================================
    // Images
    // ========================================================================

    /// Insert an `ImageNode` on the current entity (no size set).
    pub fn image_node(&mut self, handle: Handle<Image>) -> &mut Self {
        self.insert(ImageNode::new(handle))
    }

    /// Insert an `ImageNode` and set an explicit pixel size on the current entity.
    pub fn image_node_sized(
        &mut self,
        handle: Handle<Image>,
        width: f32,
        height: f32,
    ) -> &mut Self {
        self.size_px(width, height).insert(ImageNode::new(handle))
    }
}
