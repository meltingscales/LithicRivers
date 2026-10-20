# Spec Delta

## Purpose

Placeable blocks that carry state (a facing, an inventory, progress) and that the player can place, rotate, open and break; the base for chests, belts, machines, generators and power poles.

## ADDED Requirements

### Requirement: Placing a factory block
Placing a factory block item SHALL put the block on the target tile with the facing currently selected for placement, consume one item from the player's inventory, and draw the block with a glyph that shows its kind and, for directional blocks, its facing. Placement SHALL follow the same rules as placing other blocks (target must be free and passable for placement).

#### Scenario: Place a chest
- **WHEN** the player has a chest item, is in Place mode with the chest on the selected hotbar slot, and places east
- **THEN** a chest appears on the tile east of the player and the inventory has one chest fewer

#### Scenario: Occupied target
- **WHEN** the target tile already holds a factory block or a creature
- **THEN** nothing is placed and the message log says the tile is occupied

### Requirement: Rotate before placing
In Place mode, the rotate-placement keybind SHALL cycle the facing for the next placement clockwise (north, east, south, west), and the hotbar SHALL show the current facing.

#### Scenario: Choose a facing
- **WHEN** the player presses the rotate-placement key twice from the default facing north and places a directional block
- **THEN** the placed block faces south

### Requirement: Rotate after placing
A Rotate build mode SHALL be reachable through the existing build-mode toggle. In Rotate mode, a direction key SHALL rotate the directional factory block in that direction one step clockwise. Rotating a non-directional block or an empty tile SHALL do nothing and say so.

#### Scenario: Rotate a placed block
- **WHEN** a directional block east of the player faces north and the player presses the east direction key in Rotate mode
- **THEN** the block faces east

### Requirement: Breaking a factory block
Breaking a factory block SHALL remove it, give its item back to the player, and drop everything it contained as items on its tile.

#### Scenario: Break a full chest
- **WHEN** the player breaks a chest holding 10 logs
- **THEN** the chest item returns to the player's inventory and 10 logs lie on the chest's former tile

### Requirement: Chests
A chest SHALL hold up to 16 item stacks. Interacting with a chest SHALL open a transfer window showing the chest's and the player's inventories, where the player can move items in either direction.

#### Scenario: Store items
- **WHEN** the player opens a chest and moves 5 logs into it
- **THEN** the chest holds 5 logs and the player has 5 fewer

### Requirement: Chunk boundaries
Factory blocks SHALL behave identically wherever they are in the world, including next to and across chunk edges and at negative coordinates. When a factory block reads or changes a tile in a chunk that has not been generated yet, that chunk SHALL be generated exactly as it would be if the player had walked there, including its structures.

#### Scenario: Block on a chunk edge
- **WHEN** a directional block at x = 63 faces east toward x = 64 in the next chunk
- **THEN** it interacts with the block at x = 64 exactly as it would within one chunk

#### Scenario: Negative coordinates
- **WHEN** a directional block at x = 0 faces west toward x = -1
- **THEN** it interacts with the block at x = -1 exactly as it would at any other coordinates

#### Scenario: Ungenerated neighbor chunk
- **WHEN** a factory block reaches into a chunk the player has never visited
- **THEN** the chunk is generated, and its tiles and structures match the chunk generated for a player walking there in a game with the same seed
