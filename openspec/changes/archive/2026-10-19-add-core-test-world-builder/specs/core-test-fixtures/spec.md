# Spec Delta

## Purpose

Lets tests build a minimal, deterministic game scenario with only the entities they place, then drive it tick by tick and inspect the result.

## ADDED Requirements

### Requirement: Minimal scenario
A fixture built from a seed SHALL contain exactly one player and no other entities unless the test adds them. The player's position SHALL be settable.

#### Scenario: Empty scenario
- **WHEN** a test builds a fixture with seed 1 and player at (0, 0, 0) and adds nothing else
- **THEN** the game contains the player at (0, 0, 0) and no sheep, dogs, NPCs, or dropped items

### Requirement: Placing tiles
A fixture SHALL allow setting individual tiles and filling a rectangular area with one tile kind before the game starts. Placed tiles SHALL override world generation.

#### Scenario: Floor area
- **WHEN** a test fills (-5,-5,0)..(5,5,0) with grass and places a tree at (1, 0, 0)
- **THEN** the built game reports grass across the area except a tree at (1, 0, 0)

### Requirement: Placing entities and items
A fixture SHALL allow adding sheep, feral dogs, NPCs, and dropped items at given positions, and SHALL return a handle usable to look each one up after building.

#### Scenario: Dog next to player
- **WHEN** a test adds a feral dog at (2, 0, 0)
- **THEN** after building, the returned handle resolves to a feral dog at (2, 0, 0) with full health

### Requirement: Player loadout
A fixture SHALL allow replacing the player's inventory with a given item list and setting the player's auto-pickup flag, so tests do not depend on config defaults.

#### Scenario: Empty inventory
- **WHEN** a test sets the player's inventory to empty
- **THEN** the built game's player inventory contains no items

#### Scenario: Auto-pickup off
- **WHEN** a test turns auto-pickup off and the player walks onto a dropped item
- **THEN** the item stays on the ground and the inventory is unchanged

### Requirement: Driving the game
A built fixture SHALL let a test queue a player action (move, mine, mine at, interact) and run one or more ticks, returning what happened on each tick.

#### Scenario: Chop tree
- **WHEN** a test places a tree next to the player, queues mine-at on it, and runs one tick
- **THEN** the tick reports a mining success and logs and acorns are dropped at that position

### Requirement: Deterministic fixtures
Building the same fixture with the same seed and steps SHALL produce identical game state.

#### Scenario: Same fixture twice
- **WHEN** two identical fixtures are built and run for 10 ticks
- **THEN** all entity positions, health values, and inventories match

### Requirement: Available outside core tests
The fixture builder SHALL be usable from core's own tests and, when the `test-support` feature is enabled, from other crates' tests. It SHALL NOT be compiled into release builds of the game.

#### Scenario: Release build
- **WHEN** the client is built in release mode without the feature
- **THEN** the fixture builder is not compiled into the binary
