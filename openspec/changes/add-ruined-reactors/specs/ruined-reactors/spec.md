# Spec Delta

## Purpose

Ruined nuclear reactors are rare multiblock ruins left by an older world. This spec covers where they appear and what the player can take from them.

## ADDED Requirements

### Requirement: Reactors appear in the world
Worldgen SHALL place ruined reactors rarely, choosing positions and layouts only from the world seed and chunk coordinates. A reactor that spans several chunks SHALL be generated whole and identically whether its chunks are first reached by the player or by a machine.

#### Scenario: Same seed, same ruins
- **WHEN** two worlds with the same seed generate the same area
- **THEN** both have the same ruined reactors, with the same layouts, at the same positions

#### Scenario: Ruin across a chunk edge
- **WHEN** a ruin's footprint crosses into a chunk the player hasn't visited
- **THEN** that part of the ruin appears when the chunk is generated, matching a world where the player walked there first

### Requirement: Reactor parts and what they drop
Ruins SHALL be built from reactor casing, nuclear waste blocks and uranium deposits. Mining reactor casing SHALL take much longer than rock and drop reactor casing scrap. Mining nuclear waste SHALL drop nuclear waste. Mining a uranium deposit SHALL drop a uranium chunk. These drops SHALL be raw sources for crafting.

#### Scenario: Mine nuclear waste
- **WHEN** the player mines a nuclear waste block
- **THEN** the block is removed and nuclear waste drops on its tile

#### Scenario: Uranium is rare
- **WHEN** the blocks of every layout are counted
- **THEN** each layout has at least one uranium deposit and far fewer uranium deposits than nuclear waste blocks
