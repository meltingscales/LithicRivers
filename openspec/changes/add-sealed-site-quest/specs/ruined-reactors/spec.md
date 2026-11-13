# Spec Delta

## Purpose

Ruined nuclear reactors are rare multiblock ruins left by an older world. This spec covers where they appear and what the player can take from them.

## ADDED Requirements

### Requirement: Reactors appear in the world
Worldgen SHALL place ruined reactors rarely on the surface, choosing positions and layouts only from the world seed and chunk coordinates, without moving any other structure. A reactor that spans several chunks SHALL be generated whole and identically whether its chunks are first reached by the player or by a machine.

#### Scenario: Same seed, same ruins
- **WHEN** two worlds with the same seed generate the same area
- **THEN** both have the same ruined reactors, with the same layouts, at the same positions

#### Scenario: Other structures unchanged
- **WHEN** a chunk that held an outpost, temple or skyscraper before this change is generated
- **THEN** it still holds the same structure in the same place

### Requirement: Reactor parts and what they drop
Ruins SHALL be built from reactor casing, nuclear waste blocks and uranium deposits. Mining reactor casing SHALL drop reactor casing scrap, mining a nuclear waste block SHALL drop nuclear waste, and mining a uranium deposit SHALL drop a uranium chunk. These drops SHALL be raw sources for crafting.

#### Scenario: Mine nuclear waste
- **WHEN** the player mines a nuclear waste block
- **THEN** the block is removed and the player gains nuclear waste

#### Scenario: Uranium is rare
- **WHEN** the blocks of every layout are counted
- **THEN** each layout has at least one uranium deposit and far fewer uranium deposits than nuclear waste blocks

### Requirement: Hard tiles take longer to mine
Mining a tile SHALL take its hardness times the usual mining time. Reactor casing SHALL have a hardness of five; every other tile SHALL keep a hardness of one.

#### Scenario: Casing is slow
- **WHEN** the player mines reactor casing and then rock with the same body
- **THEN** the casing takes five times as long as the rock
