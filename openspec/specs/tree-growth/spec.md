# tree-growth Specification

## Purpose

Lets trees regrow: acorns are planted as saplings that grow into choppable trees over game time.

## Requirements

### Requirement: Planting acorns
Placing an acorn on a grass or dirt tile SHALL consume the acorn and create a sapling there. Planting on any other tile, or on a tile with something on it, SHALL fail with a message.

#### Scenario: Plant on grass
- **WHEN** the player places an acorn on an adjacent grass tile
- **THEN** a sapling appears there and the player has one acorn fewer

#### Scenario: Plant on stone
- **WHEN** the player tries to place an acorn on rock
- **THEN** nothing is planted and the message log says acorns need grass or dirt

### Requirement: Saplings grow into trees
A sapling SHALL become a tree once a growth time has passed since it was planted. The growth time SHALL be between 3 and 5 game minutes, determined by the world seed and the sapling's position, so the same sapling grows at the same tick in every run. Growth SHALL follow factory time and continue wherever the sapling is.

#### Scenario: Sapling grows
- **WHEN** a sapling's growth time is 4 game minutes and the player passes 4 game minutes
- **THEN** the sapling is now a tree that can be chopped for logs and acorns like any natural tree

#### Scenario: Same sapling, same timing
- **WHEN** two games with the same seed plant a sapling at the same position at the same tick
- **THEN** both saplings become trees at the same tick

### Requirement: Saplings in the world
A sapling SHALL be drawn with its own glyph, SHALL NOT block movement, and breaking it SHALL return the acorn.

#### Scenario: Break a sapling
- **WHEN** the player breaks a sapling before it has grown
- **THEN** the sapling is removed and the player gets the acorn back
