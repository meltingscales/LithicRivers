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
