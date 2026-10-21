# conveyor-belts Specification

## Purpose

Moves items between factory blocks: belts carry items in their facing direction and deliver them into chests, machines and other belts.

## Requirements

### Requirement: Belts carry items
A belt SHALL hold at most one item and SHALL move it one tile in its facing direction every half game second, onto the next belt if that belt is empty.

#### Scenario: Item travels down a line
- **WHEN** a log is on the first of three east-facing belts in a row and one game second passes
- **THEN** the log is on the third belt

#### Scenario: Corner
- **WHEN** an east-facing belt points into a south-facing belt
- **THEN** items move from the first onto the second and continue south

### Requirement: Belts deliver into chests and machines
When the tile a belt faces holds a chest or a machine that accepts the belt's item, the belt SHALL insert the item into it. If the target is full or doesn't accept the item, the item SHALL stay on the belt and items behind it SHALL wait.

#### Scenario: Belt into a chest
- **WHEN** an east-facing belt carrying a log faces a chest with free space
- **THEN** the log moves into the chest and the belt is empty

#### Scenario: Full chest
- **WHEN** the chest at the end of a belt line is full
- **THEN** the items stay on the belts in order, and resume when the chest has space

### Requirement: Loading belts
A dropped item lying on a belt tile SHALL be picked up by that belt when it is empty. The player SHALL be able to drop an item onto an adjacent belt.

#### Scenario: Drop onto a belt
- **WHEN** the player drops a log onto an adjacent empty belt
- **THEN** the belt carries the log

### Requirement: Belts across chunk edges
A belt line SHALL work the same across chunk boundaries and at negative coordinates.

#### Scenario: Line crossing a chunk edge
- **WHEN** an east-facing belt line runs from x = 62 to x = 66 with a chest at x = 67
- **THEN** items reach the chest in the same number of steps as an identical line inside one chunk

### Requirement: Belts are craftable
Wooden conveyors SHALL have a recipe whose tree ends in raw materials.

#### Scenario: Craft belts
- **WHEN** the player has the ingredients and crafts wooden conveyors
- **THEN** wooden conveyor items are added to the inventory
