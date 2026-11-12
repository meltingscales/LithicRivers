# Spec Delta

## Purpose

Sulfur vents are fuming fissures in the deep that slowly grow sulfur crystals around them, a renewable sulfur supply the player can build beside.

## ADDED Requirements

### Requirement: Placed in the deep near lava
Worldgen SHALL place sulfur vents only in the deep band, within a few tiles of lava, deterministically from the world seed and chunk.

#### Scenario: Survey
- **WHEN** the deep is surveyed for a fixed seed
- **THEN** some vents appear, each within a few tiles of lava, and none appear in other bands

#### Scenario: Same world, same vents
- **WHEN** two worlds with the same seed generate the same chunk
- **THEN** their vents are in the same places

### Requirement: Vents can't be broken
Mining or breaking a sulfur vent SHALL leave it in place and say it can't be broken.

#### Scenario: Mining a vent
- **WHEN** the player mines a sulfur vent
- **THEN** the vent remains and the message log says it can't be broken

### Requirement: Vents grow sulfur crystals
While the player is nearby, a vent SHALL grow a sulfur crystal tile on an open, walkable tile beside it at a fixed interval, up to a cap per vent, choosing the tile from the world seed, the vent's position and the tick. Mining a crystal SHALL give sulfur and free its tile to grow again.

#### Scenario: Crystals appear
- **WHEN** the player waits beside a vent with open ground around it for several growth intervals
- **THEN** sulfur crystals have grown on tiles beside the vent

#### Scenario: Growth stops at the cap
- **WHEN** a vent's crystals reach the cap
- **THEN** no more grow until one is mined

#### Scenario: Mining a crystal
- **WHEN** the player mines a sulfur crystal
- **THEN** they gain sulfur and the tile is open again
