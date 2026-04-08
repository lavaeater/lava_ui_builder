# Lava UI Builder

The purpose of this crate is to enable code-based UI building with the builder pattern and reasonable defaults and themes.

It should be EASY to create a simple menu to start the game, make a choice in a game or display some game info.

## Issues & Simplification TODOs

### Bugs / Correctness

1. add_themed_button_observe has hardcoded "PLAAAY" as button text (builder.rs:762)                                                                   
   The function accepts a closure f but creates the text entity with Text::new("PLAAAY") instead of taking a text parameter like add_button_observe does.
   The text can only be fixed via the ButtonBuilder closure workaround. Add a text parameter and remove the hardcoded string.

2. WidgetsButton inserted twice in add_themed_button_observe (builder.rs:772, 778)                                                             
   Two separate .insert() calls on the same entity both include WidgetsButton. The second block (lines 776–786) duplicates components from the first   
   block (lines 769–774). Merge into one insert.

3. feathers_button_with_overrides creates a ButtonBuilder with text_entity: None (button_builder.rs:162)                                              
   Calling .text(), .font_size(), or .text_color() on this builder silently does nothing. The feathers button() spawns text as a child, so there's no  
   handle to it — but this should either be documented clearly or the text entity should be found after spawning.

4. build() returns wrong entity when parent_stack is non-empty (builder.rs:123)                                                                       
   Returns parent_stack[0] (the oldest ancestor) rather than the actual root that was spawned in new(). Store the root entity explicitly at construction
   time instead.

  ---                                                                                               
### Duplication

5. Three nearly-identical button setup methods: add_themed_button, add_button_observe, add_themed_button_observe
   All three: spawn a child, build a Node + colors + InteractionPalette, spawn a text child, create a ButtonBuilder, run a panic-safe closure, restore   
   stack. Extract a private spawn_button_inner(text, node, extras, handler, f) helper that all three delegate to.

6. spawn_collapsible_section in systems.rs duplicates the collapsible logic from with_collapsible in builder.rs                                       
   Two separate implementations of the same widget. Remove spawn_collapsible_section (or make with_collapsible call a shared internal function it also
   exposes).

7. flex_dir_row / flex_dir_column / flex_direction_row are aliases for flex_row / flex_column (builder.rs:384–392)                                    
   Three methods, one behaviour. Remove the aliases and keep only flex_row / flex_column.

8. add_collapsible / add_collapsible_collapsed just call with_collapsible with a boolean (builder.rs:856–866)                                         
   Not worth two separate methods — the boolean makes intent clear enough. Remove them or at least keep only with_collapsible.

  ---                                                                                                                                                 
### Hardcoded values that should live in the theme

9. Collapsible toggle button colors are hardcoded in three places
- builder.rs:837: Color::srgb(0.25, 0.25, 0.3) (normal)
- systems.rs:50: Color::srgb(0.4, 0.6, 0.4) (pressed)
- systems.rs:56: Color::srgb(0.35, 0.35, 0.4) (hovered)
- systems.rs:58/157: same normal color again in spawn_collapsible_section

Add a collapsible_bg, collapsible_bg_hovered, collapsible_bg_pressed to ButtonTheme (or a dedicated CollapsibleTheme) and read from the resource in   
the system.

10. add_themed_button_observe hardcodes width: 150px, height: 75px (builder.rs:748–749)                                                               
    These should come from theme.button.width / theme.button.height, just like add_button_observe uses button_theme.border_width.

11. adapt_ui_scale hardcodes base width 1920.0 (lib.rs:361)                                                                                           
    Move this to LavaTheme.ui_width (the field already exists) so users can configure it.

  ---                                                                                                                                                 
### Minor

12. with_text has six Option parameters — use a builder or a struct instead
    with_text(text, None, None, None, None, None) is unreadable. Introduce a TextStyle struct with defaults, or at minimum provide named convenience      
    constructors.

13. VecDeque import in builder.rs is redundant — VecDeque is only used in lib.rs                                                                      
    use std::collections::VecDeque; at the top of builder.rs re-imports a type that belongs to lib.rs. Remove it.