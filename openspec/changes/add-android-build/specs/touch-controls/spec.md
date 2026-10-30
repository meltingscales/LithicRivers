# Spec Delta

## Purpose

Playing without a keyboard: on-screen controls that send the same keys as the keyboard, tap and drag input, and the soft keyboard for text fields.

## ADDED Requirements

### Requirement: On-screen controls
On a touch screen, the game window SHALL show an on-screen pad with the eight movement directions and wait, and buttons for Interact, Mine, Build mode, the previous and next menu tab, and Escape. Pressing one SHALL send the key bound to that action, so keybinds and every screen work as with a keyboard. The pad SHALL be shown by default on touch devices, and an Options setting SHALL show or hide it and be saved.

#### Scenario: Moving with the pad
- **WHEN** the player taps the pad's east arrow
- **THEN** the robot moves east, exactly as pressing the key bound to `movement.MOVE_EAST` would

#### Scenario: Rebound key
- **WHEN** the player has rebound Interact to another key and taps the Interact button
- **THEN** the game receives the rebound key and interacts

#### Scenario: Hiding the pad
- **WHEN** the player hides the on-screen controls in Options
- **THEN** the pad disappears, the game uses the freed space, and the choice persists

### Requirement: Tap and drag
Tapping a cell outside the on-screen controls SHALL act as a left click on that cell, and dragging SHALL scroll, one scroll step per cell dragged, like the mouse wheel.

#### Scenario: Tapping a menu row
- **WHEN** the player taps a row in the Options screen
- **THEN** the row is selected as a mouse click would select it

#### Scenario: Dragging a list
- **WHEN** the player drags upward over the inventory list
- **THEN** the list scrolls as it would with the mouse wheel

### Requirement: Soft keyboard for text
When a text field takes typing (the world name and seed, a search, the cheat console), the soft keyboard SHALL open, and its characters SHALL reach the field as typed keys. It SHALL close when the field stops taking typing.

#### Scenario: Naming a world
- **WHEN** the player selects the world name field on a phone
- **THEN** the soft keyboard opens, and the typed name appears in the field
