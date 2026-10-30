# touch-controls Specification

## Purpose
Playing without a keyboard: on-screen controls that send the same keys as the keyboard, tap and drag input, and the soft keyboard for text fields.

## Requirements

### Requirement: On-screen controls
On mobile builds (phones and tablets), the game window SHALL show an on-screen pad floating over the game with the eight movement directions and wait, and buttons for Interact, Mine, Build mode, the previous and next menu tab, Enter and Escape. Pressing one SHALL send the key bound to that action, so keybinds and every screen work as with a keyboard. The pad SHALL be shown by default on mobile, and an Options setting SHALL show or hide it and be saved.

#### Scenario: The game shows through
- **WHEN** the pad is shown
- **THEN** the game keeps the whole screen, and under each button its text shows through dimmed

#### Scenario: Moving with the pad
- **WHEN** the player taps the pad's east arrow
- **THEN** the robot moves east, exactly as pressing the key bound to `movement.MOVE_EAST` would

#### Scenario: Rebound key
- **WHEN** the player has rebound Interact to another key and taps the Interact button
- **THEN** the game receives the rebound key and interacts

#### Scenario: Hiding the pad
- **WHEN** the player hides the on-screen controls in Options
- **THEN** the pad disappears from over the game, and the choice persists

### Requirement: Touch on mobile only
Desktop builds assume a keyboard: they SHALL NOT show the pad or take touch input, whatever the setting says.

#### Scenario: Desktop
- **WHEN** the game runs on Windows, Linux or macOS, even on a touch screen
- **THEN** no pad is shown, and Options shows "Touch controls: only on mobile"

### Requirement: Tap and drag
On mobile, tapping a cell outside the on-screen controls SHALL act as a left click on that cell, and dragging SHALL scroll, one scroll step per cell dragged, like the mouse wheel.

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
