# Spec Delta

## Purpose

The Choose Interaction menu as a 3x3 grid of the tiles around the robot, chosen by direction.

## ADDED Requirements

### Requirement: The grid
When Interact finds more than one choice, the Choose Interaction menu SHALL show a 3x3 grid of the robot's tile and its eight neighbors, laid out as on the map, with the robot in the middle. Each cell SHALL show the printable key bound to that direction (the wait key for the middle) and the glyph of the thing there, dimmed when the cell has no choice. A list beside the grid SHALL name every choice with its key in brackets.

#### Scenario: A chest and a corpse
- **WHEN** a chest is west of the robot, a corpse is east, and the player presses Interact
- **THEN** the menu shows the chest's glyph in the 4 cell and the corpse's in the 6 cell, and the list reads "[4] Open chest" and "[6] Loot corpse (2 items)"

### Requirement: Choosing by direction
Pressing a movement key SHALL choose the tile in that direction, and the wait key the robot's own tile. If that tile has one choice it SHALL run at once; with several, the first press SHALL select the first, each further press SHALL move to the next, and Enter SHALL run the selected one. A direction with no choice SHALL do nothing but say so.

#### Scenario: One press
- **WHEN** the menu shows a chest to the west and the player presses the west movement key
- **THEN** the chest window opens

#### Scenario: Two choices on one tile
- **WHEN** a corpse and a dropped item are both east of the robot and the player presses east twice, then Enter
- **THEN** the second of the two choices runs

#### Scenario: Empty direction
- **WHEN** the player presses the north key and nothing is north
- **THEN** the menu stays open and the message log says there is nothing there

### Requirement: Moving through the list
The Up and Down keys (`ui.MENU_UP`, `ui.MENU_DOWN`) SHALL move through every choice in the list, and the grid SHALL highlight the selected choice's cell.

#### Scenario: Arrow keys
- **WHEN** the menu lists three choices and the player presses Down twice, then Enter
- **THEN** the third choice runs

### Requirement: Mouse on the grid
Clicking a grid cell SHALL act as pressing its direction key, and clicking a list row SHALL choose that row.

#### Scenario: Click a cell
- **WHEN** the player clicks the cell holding the chest
- **THEN** the chest window opens
