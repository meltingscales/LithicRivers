# Spec Delta

## Purpose

A powered compass that finds the nearest biome of a chosen kind and can mark it on the Global Map.

## ADDED Requirements

### Requirement: Finding a biome
Using the biome compass on a chosen biome kind SHALL report the direction and distance to the nearest one within 8,000 tiles, or that none is in range, and SHALL use a fixed charge from a carried power cell either way.

#### Scenario: Found
- **WHEN** the player searches for a Fractal Tear with a charged cell, and one exists 3,000 tiles away
- **THEN** the compass reports its direction, distance and depth, and the cell's charge drops

#### Scenario: No charge
- **WHEN** every carried cell is empty
- **THEN** the compass doesn't search and says there is no charge

### Requirement: Marking the result
The player SHALL be able to mark a found biome as a waypoint on the Global Map.

#### Scenario: Mark
- **WHEN** the player chooses "Mark it" after a search
- **THEN** a waypoint named after the biome appears on the Global Map at that spot

### Requirement: Every biome is findable
Every biome kind that worldgen can generate SHALL be in the compass's list.

#### Scenario: A new biome
- **WHEN** a biome is added to worldgen without adding it to the compass's list
- **THEN** a test fails naming it
