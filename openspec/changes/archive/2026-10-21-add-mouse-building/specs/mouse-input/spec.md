# Spec Delta

## Purpose

Point-and-click building and navigation with the mouse, as an optional alternative to the keyboard.

## ADDED Requirements

### Requirement: Clicking world tiles
On the World tab, a left click on a tile within reach (up to 5 tiles from the player) SHALL place the selected hotbar block there (when it holds a placeable item), otherwise interact with the block there, otherwise look at the tile; a right click within reach SHALL break or mine it. Clicks beyond reach SHALL only look at the tile and say it is too far.

#### Scenario: Place with the mouse
- **WHEN** a chest is selected in the hotbar and the player left-clicks the grass tile east of them
- **THEN** a chest is placed there, as with the place-east key

#### Scenario: Too far
- **WHEN** the player right-clicks a tree six tiles away
- **THEN** nothing is mined, the look panel shows the tree, and a message says it is too far

### Requirement: Scroll and rotate
The scroll wheel SHALL cycle the selected hotbar slot, and the rotate binding (Shift + scroll) SHALL rotate the next placement.

#### Scenario: Scroll the hotbar
- **WHEN** the player scrolls down once on the World tab
- **THEN** the next hotbar slot is selected

### Requirement: Pick block
A middle click on a placed block SHALL select the hotbar slot holding that block's item (or assign the item to the current slot if the player carries it) and SHALL copy the block's facing to the next placement. It SHALL work at any distance and change nothing in the world.

#### Scenario: Pick a belt
- **WHEN** the player middle-clicks a west-facing belt while carrying wooden conveyors
- **THEN** wooden conveyors are selected in the hotbar and the next placement faces west

#### Scenario: Nothing to pick
- **WHEN** the player middle-clicks a chest without carrying any chests
- **THEN** the selection doesn't change and a message says they have no chest

### Requirement: Hover highlight
The tile under the mouse SHALL be highlighted on the World tab, and in Place mode SHALL show a ghost of the selected block with its facing, without changing the game.

#### Scenario: Ghost in Place mode
- **WHEN** the mouse rests over a free tile in Place mode with a belt selected
- **THEN** the tile shows a dim belt facing the current placement facing

### Requirement: Menu and list clicks
Clicking a tab in the menu bar SHALL switch to it, and the scroll wheel SHALL scroll lists such as the Item Dictionary, crafting recipes and cheat console suggestions.

#### Scenario: Click a tab
- **WHEN** the player clicks "Items" in the menu bar
- **THEN** the Item Dictionary tab opens

### Requirement: Mouse bindings and headless steps
Mouse actions SHALL be bound in the config like keys, and headless scripts SHALL support `click`, `scroll` and `move` steps that behave exactly like real mouse events.

#### Scenario: Scripted click
- **WHEN** a script runs `click 52 14 left` over a tile next to the player in Place mode
- **THEN** the block is placed as if the player had clicked there
