# Spec Delta

## Purpose

Molten lava fills the Lithic Rivers. This spec covers what lava does to movement, mining, sight and block placement.

## ADDED Requirements

### Requirement: Lava blocks movement
Lava SHALL be impassable to the player and to creatures, and SHALL NOT be mineable.

#### Scenario: Walking into lava
- **WHEN** the player tries to move onto a lava tile
- **THEN** the player stays where they are

#### Scenario: Mining lava
- **WHEN** the player mines a lava tile
- **THEN** the tile stays lava and nothing drops

### Requirement: Sight passes over lava
Lava SHALL NOT block line of sight or light.

#### Scenario: Seeing across a river
- **WHEN** a lava tile lies between the player and a tile within their light radius
- **THEN** that tile is lit

### Requirement: Blocks bridge lava
Placing a plain block on lava SHALL use up the block and turn the tile into walkable lava crust. Factory blocks and blueprint pastes SHALL NOT be placed on lava.

#### Scenario: Making a crossing
- **WHEN** the player places a stone on a lava tile next to them
- **THEN** one stone is used, the tile becomes lava crust, and the player can walk onto it

#### Scenario: No machines on lava
- **WHEN** the player tries to place a chest on a lava tile
- **THEN** nothing is placed and the chest stays in the inventory
