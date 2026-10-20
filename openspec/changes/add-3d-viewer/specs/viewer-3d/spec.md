# Spec Delta

## Purpose

An optional window that shows the game world around the player in simple 3D, for viewing only, alongside the ASCII game.

## ADDED Requirements

### Requirement: Opening the viewer
The player SHALL be able to open the 3D viewer from the in-game Menu tab (and from the title screen's Options when it exists). It SHALL open in its own window while the terminal game keeps running and keeps all input. Opening it again while it is open SHALL bring the existing window forward instead of opening a second one.

#### Scenario: Open from the Menu tab
- **WHEN** the player selects Open 3D Viewer in the Menu tab
- **THEN** a separate window opens showing the area around the player, and the terminal game keeps responding to keys

### Requirement: Viewer shows the world around the player
The viewer SHALL draw the tiles in a box around the player, several z-layers deep, as blocks colored by tile kind, and creatures, items and factory blocks as simple shapes, updating whenever the game state changes. The camera SHALL follow the player; the mouse SHALL orbit and zoom around the player.

#### Scenario: Follows the player
- **WHEN** the player moves three tiles east in the terminal game
- **THEN** the viewer camera moves to keep the player centered

#### Scenario: Underground layers
- **WHEN** the player is in the bunker underground
- **THEN** the viewer shows the layers above and below the player as blocks, with the player's layer cut open so the player is visible

### Requirement: Tooltips on hover
Hovering the mouse over a block or entity in the viewer SHALL show a tooltip with its name and, for factory blocks, its state (for example, a chest's item count or a machine's status).

#### Scenario: Hover a dog
- **WHEN** the mouse hovers over a feral dog
- **THEN** a tooltip shows "Feral Dog" and its health

### Requirement: Simple animations
When an entity moves between tiles, the viewer SHALL animate it sliding to its new position over a short time instead of jumping; items on belts SHALL slide along the belt the same way. No other animation is required.

#### Scenario: Movement glides
- **WHEN** the player moves one tile
- **THEN** the player's shape slides to the new tile over a fraction of a second

### Requirement: The viewer never affects the game
The viewer SHALL be read-only: it SHALL NOT change game state, and closing it, failing to start it (for example, no compatible GPU), or the viewer crashing SHALL NOT interrupt or alter the game. A failure to start SHALL be reported in the message log.

#### Scenario: No GPU
- **WHEN** the player opens the viewer on a machine without a compatible graphics adapter
- **THEN** the message log says the 3D viewer couldn't start, and the game continues

#### Scenario: Viewer closed mid-game
- **WHEN** the player closes the viewer window
- **THEN** the game continues unchanged and the viewer can be opened again

#### Scenario: Determinism unaffected
- **WHEN** the same headless script runs with and without the viewer open
- **THEN** the game state is identical
