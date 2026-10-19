# Spec Delta

## Purpose

Gives agents and tooling a compact, deterministic JSON view of the current game state, so they can reason about the game without parsing ASCII screens.

## ADDED Requirements

### Requirement: State dump step
`state [label]` in a headless script SHALL print one JSON object between `=== state <label> ===` and `=== end state ===` lines.

#### Scenario: State dump framing
- **WHEN** a script runs `state after-move`
- **THEN** stdout contains `=== state after-move ===`, a single JSON object, and `=== end state ===`

### Requirement: Snapshot contents
The snapshot SHALL include: format version, seed, current tick, player position, player body part states and integrity, inventory items with quantities, active UI tab and modal, whether combat is active and its participants, entities within the configured radius of the player (kind, position, health where applicable), and the most recent messages from the message log.

#### Scenario: Player and inventory present
- **WHEN** a new game with seed 12345 has started and the script runs `state`
- **THEN** the JSON contains seed 12345, the player's position, and the starting inventory items with their quantities

#### Scenario: Nearby entity listed
- **WHEN** an entity is within the snapshot radius of the player
- **THEN** it appears in the nearby-entities list with its kind and position

### Requirement: Snapshot is bounded
The snapshot SHALL NOT include tile or chunk data, and SHALL cap the nearby-entity list and message list at fixed maximum lengths, so its size does not grow with world exploration.

#### Scenario: Large explored world
- **WHEN** the player has explored many chunks
- **THEN** the snapshot contains no tile data and its entity and message lists stay within their caps

### Requirement: Deterministic serialization
Snapshot serialization SHALL be deterministic: object keys in a fixed order and list entries sorted by a stable key (entities by distance, then position, then kind; inventory by item name; messages by tick order).

#### Scenario: Same state, same bytes
- **WHEN** the same game state is snapshotted twice
- **THEN** both JSON outputs are byte-identical
