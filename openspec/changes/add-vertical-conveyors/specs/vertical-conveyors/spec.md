# Spec Delta

## Purpose

Move items between layers, so mines and factories can stack.

## ADDED Requirements

### Requirement: Vertical conveyors move items between layers
A vertical conveyor SHALL take items like a belt and, one belt-tile time later, hand each to the block directly above it (going up) or below it (going down), if that block accepts it; otherwise the item SHALL wait.

#### Scenario: Lift into a chest
- **WHEN** a belt carries a log into an up conveyor at z = 0 with a chest directly above it at z = 1
- **THEN** the log ends up in the chest

#### Scenario: Stacked conveyors
- **WHEN** three up conveyors are stacked at z = 0, 1 and 2 with a chest at z = 3
- **THEN** items fed into the bottom one reach the chest

### Requirement: Up or down
A placed vertical conveyor SHALL go up, and rotating it SHALL toggle between up and down.

#### Scenario: Toggle
- **WHEN** the player rotates an up conveyor
- **THEN** it goes down, and its glyph shows it

### Requirement: Solid layers block the lift
If the tile above or below is solid terrain, the vertical conveyor SHALL hold its item and lose nothing.

#### Scenario: Under rock
- **WHEN** an up conveyor has solid rock directly above it
- **THEN** its item stays on it

### Requirement: Recipes and conversion
Vertical conveyors SHALL be craftable from the same ingredients as wooden conveyors, and SHALL convert to and from wooden conveyors one for one.

#### Scenario: Convert
- **WHEN** the player crafts the conversion with one wooden conveyor
- **THEN** they get one vertical conveyor

### Requirement: Climbing vertical conveyors
The player SHALL be able to move up or down from a vertical conveyor or stairs to a vertical conveyor or stairs directly above or below, as with stairs.

#### Scenario: Climb a stack
- **WHEN** the player stands on a vertical conveyor with another directly above it and moves up
- **THEN** they are on the upper conveyor, and moving down brings them back

### Requirement: Building stairs
Stairs SHALL be craftable, and placing them SHALL build a stairway of two stair tiles, at the target and directly above or below it (the player's choice), using one Stairs item per tile. A solid, mineable tile on the other layer SHALL be dug out with its usual drops. Breaking a stair tile SHALL give one Stairs item.

#### Scenario: Dig a stairway up
- **WHEN** the player with 2 Stairs places them next to them under rock
- **THEN** stairs appear at the target and above it, the rock's stone drops on the upper stairs, and the player can climb up and back down

#### Scenario: Too few stairs
- **WHEN** the player places stairs with only 1 in the inventory
- **THEN** nothing is built and the message says a stairway takes 2

### Requirement: Ghost stairs
On every other second, a bare tile with stairs or a vertical conveyor directly above or below it SHALL show a dim ghost of the stairs.

#### Scenario: Above a stairway
- **WHEN** the player looks at the layer above a stairway's top
- **THEN** the tile over it shows a ghost of the stairs every other second
