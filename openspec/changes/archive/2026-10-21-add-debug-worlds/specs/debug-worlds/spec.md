# Spec Delta

## Purpose

Debug world types give developers and curious players ready-made places to test mechanics: empty space, every ore, a crowd of enemies, and working example factories.

## ADDED Requirements

### Requirement: Choosing a world type
The Create New World screen SHALL offer a World type choice (Normal, Void, Ore test, Dog arena, Factory showcase) when Developer mode is on, defaulting to Normal. With Developer mode off, the choice SHALL be hidden and the world SHALL be Normal. The type SHALL be saved with the world and shown in the world list for every type except Normal.

#### Scenario: Hidden without developer mode
- **WHEN** the player creates a world with Developer mode off
- **THEN** no World type row is shown and the world is Normal

#### Scenario: Shown in the list
- **WHEN** a Void world named "Lab" is saved
- **THEN** the world list shows it as "Lab (Void)"

### Requirement: Debug worlds are reproducible
Every world type SHALL generate the same world from the same seed, including everything it places (ore rows, dogs, showcase builds).

#### Scenario: Same seed, same arena
- **WHEN** two Dog arena worlds are created with seed 7
- **THEN** both have the same 100 dogs at the same positions

### Requirement: Void
A Void world SHALL contain no terrain, structures, creatures or spouts, except a small grass platform under the spawn point.

#### Scenario: Look around
- **WHEN** a Void world starts
- **THEN** the player stands on grass, every tile beyond the platform is empty, and there are no creatures

### Requirement: Ore test
An Ore test world SHALL lay out, near the spawn point on flat ground, one row for every tile in the mining drop table, so a tile added to the table later appears without other changes.

#### Scenario: Every minable tile
- **WHEN** an Ore test world starts
- **THEN** every tile that the mining drop table lists appears in its own row near spawn

### Requirement: Dog arena
A Dog arena world SHALL be a normal world with 100 feral dogs placed in rings around the spawn point, none closer than 8 tiles.

#### Scenario: Count the dogs
- **WHEN** a Dog arena world starts
- **THEN** it has exactly 100 more feral dogs than a Normal world with the same seed, all at least 8 tiles from spawn

### Requirement: Factory showcase
A Factory showcase world SHALL build, on flat ground near spawn, a set of working example setups ordered from simple to complex. Every factory block kind in the game SHALL appear in at least one example; a test SHALL enforce this, so each new factory block must add an example.

#### Scenario: Coverage
- **WHEN** a developer adds a factory block kind without a showcase example
- **THEN** `cargo test` fails and names the kind

#### Scenario: Examples run
- **WHEN** a Factory showcase world passes 60 game seconds
- **THEN** every example with an input source has made progress (items moved or produced)
