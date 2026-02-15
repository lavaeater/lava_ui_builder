use bevy::ecs::spawn::Spawn;
use bevy::ecs::system::IntoObserverSystem;
use bevy::prelude::*;

use bevy::feathers::controls::{button, ButtonProps, ButtonVariant};
use bevy::feathers::rounded_corners::RoundedCorners;
use bevy::feathers::theme::ThemedText;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;

use crate::{ButtonBuilder, UIBuilder};

// ============================================================================
// ButtonBuilder — for imperative button customization via add_themed_button()
// ============================================================================

impl<'a, 'w, 's> ButtonBuilder<'a, 'w, 's> {
    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).insert(Text::new(text.into()));
        }
        self
    }

    pub fn font_size(&mut self, size: f32) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).entry::<TextFont>()
                .and_modify(move |mut tf| { tf.font_size = size; });
        }
        self
    }

    pub fn text_color(&mut self, color: Color) -> &mut Self {
        if let Some(text_entity) = self.text_entity {
            self.ui.commands.entity(text_entity).insert(TextColor(color));
        }
        self
    }

    pub fn width_px(&mut self, width: f32) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.width = Val::Px(width); });
        self
    }

    pub fn height_px(&mut self, height: f32) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.height = Val::Px(height); });
        self
    }

    pub fn size_px(&mut self, width: f32, height: f32) -> &mut Self {
        self.width_px(width).height_px(height)
    }

    pub fn bg_color(&mut self, color: Color) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(BackgroundColor(color));
        self
    }

    pub fn border_radius_all_px(&mut self, radius: f32) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).entry::<Node>()
            .and_modify(move |mut n| { n.border_radius = BorderRadius::all(Val::Px(radius)); });
        self
    }

    pub fn insert<T: Component>(&mut self, component: T) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(component);
        self
    }

    pub fn disabled(&mut self) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).insert(InteractionDisabled);
        self
    }

    pub fn enabled(&mut self) -> &mut Self {
        self.ui.commands.entity(self.ui.current_entity).remove::<InteractionDisabled>();
        self
    }
}

// ============================================================================
// Feathers-style button helpers using observe() pattern
// ============================================================================

impl<'w, 's> UIBuilder<'w, 's> {
    /// Add a Feathers-style button with an observer for the Activate event.
    pub fn feathers_button<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_overrides(text, ButtonProps::default(), (), handler, |_| {})
    }

    /// Add a Feathers-style button with custom props and an observer.
    pub fn feathers_button_with_overrides<B, M, F>(
        &mut self,
        text: impl Into<String>,
        props: ButtonProps,
        overrides: B,
        handler: impl IntoObserverSystem<Activate, (), M>,
        f: F,
    ) -> &mut Self
    where
        B: Bundle,
        F: FnOnce(&mut ButtonBuilder),
    {
        let text_str = text.into();
        let original_entity = self.current_entity;
        let original_stack_len = self.parent_stack.len();

        self.child();
        let button_entity = self.current_entity;
        let btn = &self.theme.button;
        let button_bundle = button(
            props,
            overrides,
            Spawn((Text::new(text_str), ThemedText)),
        );

        let border_radius = self.theme.button.border_radius;

        self.commands
            .entity(button_entity)
            .insert(button_bundle)
            .observe(handler)
            .entry::<Node>()
            .and_modify(move |mut n| { 
                n.border_radius = border_radius;
                
            });

        let mut button_builder = ButtonBuilder {
            ui: self,
            text_entity: None,
        };
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut button_builder)));

        self.current_entity = original_entity;
        while self.parent_stack.len() > original_stack_len {
            self.parent_stack.pop_back();
        }

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }

        self
    }

    /// Add a Feathers-style primary button with an observer.
    pub fn feathers_button_primary<M, F>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
        f: F,
    ) -> &mut Self
    where
        F: FnOnce(&mut ButtonBuilder),
    {
        self.feathers_button_with_overrides(
            text,
            ButtonProps {
                variant: ButtonVariant::Primary,
                ..default()
            },
            (),
            handler,
            f,
        )
    }

    /// Add a disabled Feathers-style button with an observer.
    pub fn feathers_button_disabled<M>(
        &mut self,
        text: impl Into<String>,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_overrides(text, ButtonProps::default(), InteractionDisabled, handler, |_| {})
    }

    /// Add a Feathers-style button with a marker component and an observer.
    pub fn feathers_button_marked<T, M>(
        &mut self,
        text: impl Into<String>,
        marker: T,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self
    where
        T: Component,
    {
        self.feathers_button_with_overrides(text, ButtonProps::default(), marker, handler, |_| {})
    }

    /// Add a Feathers-style button with custom rounded corners.
    pub fn feathers_button_corners<M>(
        &mut self,
        text: impl Into<String>,
        corners: RoundedCorners,
        handler: impl IntoObserverSystem<Activate, (), M>,
    ) -> &mut Self {
        self.feathers_button_with_overrides(
            text,
            ButtonProps {
                corners,
                ..default()
            },
            (),
            handler,
            |_| {},
        )
    }
}
