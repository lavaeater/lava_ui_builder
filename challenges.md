# Challenges

## 1. Game Menu

![Game menu example (Factorio)](https://github.com/bevyengine/bevy/assets/13908946/02d905ef-46ee-43a7-9e84-a31045c83ba8)

### Motivation

Menus are needed for almost every game.
Here we can showcase basic input functionality like buttons, sliders, and dropdowns.

### Acceptance Criteria

- Main menu with sub menus for audio and graphics.
- Simple buttons for option selection.
- Slider for volume.
- Dropdown for graphics quality (low/medium/high).
- Navigation possible with mouse, keyboard and controller.
    - Mouse: Separate styles for hover and press.
    - Keyboard/Controller: Separate styles for currently focused element.

## 2. Inventory

![Inventory example (Minecraft)](https://github.com/bevyengine/bevy/assets/13908946/edebeaeb-de9f-4b3d-b848-489b6f1f5f83)

### Motivation

Inventories are also important in a broad range of game genres.
This can be used to test drag and drop, as well as a combination of image and text elements.
Furthermore, a tooltip implementation is tested.

### Acceptance Criteria

- Fixed-size grid, some spaces with items and some empty.
- Each item slot has an image of the item and the item count overlayed on the image.
- Items can be moved with drag and drop.
    - Both image and item count move along with the cursor while dragging.
    - The image and item count are not visible in the original position while dragging.
    - You can leave the bounding box of the inventory while dragging.
- A tooltip with the item's name is shown when hovering over an item.

## 3. Health Bar

![Health bar example (League of Legends)](https://github.com/bevyengine/bevy/assets/13908946/a0af744a-8317-4e14-9e5e-f2b053a78590)

### Motivation

Again, widely used feature in multiple genres.
Uses world-space UI and an information flow from the game to UI.

### Acceptance Criteria

- Simple 3D Scene with a character (sphere).
    - Can be moved around with WASD/arrow keys.
- A health bar and character name is anchored to the character in world-space.
- The health starts at 10 and decreases by 1 every second. The health should be stored and managed in Bevy ECS.
- When reaching 0 HP, the character should be despawned together with UI.

## 4. Responsive Menu

### Motivation

A combination of responsive design, for games which need to save multiple display sizes and formats. And nine-patch UI, which is a common styling solution in games.

### Acceptance Criteria

- A simple game menu, with buttons that use a nine-patch system for design (i.e., composed of images for the corners and middle segments) and an image to the right of the buttons.
- For normal screen sizes, the menu is centered in the middle of the screen
- For 400px width and lower, the buttons fill the screen width and the image is above the buttons.

## 5. Character Editor

![Character editor example (Sims 4)](https://github.com/bevyengine/bevy/assets/13908946/bd177192-79dd-451b-870e-81aec040453d)

### Motivation

This example showcases how a 3D scene can be integrated in the UI as well as having an information flow from UI to 3D scene.
Additionally, we feature a simple text input and a scroll box.

### Acceptance Criteria

- A UI on the right with a 3D scene of the character on the left.
    - The character can be simple 3D shapes.
- The UI is composed of multiple buttons to select options.
    - The selected option is highlighted.
    - There are too many buttons to fit in the box, so the box can be scrolled vertically. You can duplicate buttons or choose a small box size to simulate this.
- Changing the selection in the UI changes the 3D shapes in the 3D scene.
- On the top of the UI is a text field for the character name.

## 6. HUD

![HUD example from CS:GO](https://github.com/bevyengine/bevy/assets/13908946/985cc213-c80a-4800-a247-08b3db72be51)

### Motivation

Needed in many games.
Showcases how UI can be overlayed on top of 3D scenes and how it can be aligned in the different corners.

### Acceptance Criteria

- Top left: Image (minimap).
- Bottom left: HP counter and HP bar.
- Bottom center: Game time and scores of both teams.
- Bottom right: Ammo counter and indicator.

## 7. Bug Report Form

![Bug report form example (Windy Kingdom)](https://github.com/bevyengine/bevy/assets/13908946/e26d1833-2d97-40fa-84e8-87c874783427)

### Motivation

This example might not be as widely applicable, but showcases complex text input.

### Acceptance Criteria

- A dropdown for the type of bug (UI/cosmetics/gameplay).
- A one-line text input for the bug title.
- A multi-line text input for the bug description.
- The text editing should have the following features:
    - Cursor, which can be moved with arrow keys and mouse click.
    - Text selection.
    - Copy/paste/cut with the usual shortcuts.

## 8. Scoreboard

![Scoreboard example (CS:GO)](https://github.com/bevyengine/bevy/assets/13908946/3a82657e-4d0b-436e-bb3e-82199326d440)

### Motivation

Another commonly used UI element, which features a grid layout and a mix of images and text.

### Acceptance Criteria

- Grid with players as rows and avatar, name, kills (K), deaths (D), assists (A) as headings.
- Avatars should be images.
- Grid should be sized dynamically (i.e. to fit the names).
- Grid content (e.g. player names, K/D/A) should reside in the Bevy ECS.

## 9. Dark/Light Theme

### Motivation

Having theming options makes it easier to adjust the design of the UI at later stages of the development, avoiding hard-coding the style properties.

### Acceptance Criteria

- A simple UI layout (e.g. main menu).
- A button to toggle between light and dark theme.
- The theme should change all colors in the UI.

## 10. Design Specification

### Motivation

Most of the other example feature on widgets and features, but of course styling is a major part of UI.
In this challenge, a design specification should be followed as closely as possible.

### Acceptance Criteria

I didn't find a good open source design specification yet, please let me know if you find one!