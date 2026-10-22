# Spec Delta

## Purpose

The world's vertical layout: a sky above the surface, shallows and deep caverns below it, connected caves, natural stairways, and resource deposits that depend on depth.

## ADDED Requirements

### Requirement: Depth bands
Worldgen SHALL generate open sky above the surface (z ≥ 1), the surface biomes at z = 0 exactly as before this change, rock with tunnels and early deposits in the shallows, and the Lithic Rivers caverns in the deep.

#### Scenario: Sky and shallows
- **WHEN** a new world is generated
- **THEN** layer z = 1 is open air and layer z = -2 is mostly rock with tunnels

#### Scenario: Surface unchanged
- **WHEN** layer z = 0 of a new world is generated for a seed
- **THEN** every tile matches the generator from before this change

### Requirement: Connected underground
Caves on neighboring layers SHALL mostly overlap, natural stairways SHALL connect some of them, and a route of natural stairways SHALL lead from the starting bunker to the surface.

#### Scenario: Escape the bunker
- **WHEN** a new world starts in the bunker at z = -19
- **THEN** the player can walk and climb to z = 0 without building anything

### Requirement: Deposits
Resource deposits SHALL generate as clusters with a density set per depth band, and every deposit tile SHALL be solid and mineable with a drop.

#### Scenario: Enough electronics
- **WHEN** the shallows near spawn are surveyed for a fixed seed
- **THEN** scrap electronics make up at least the pinned share of tiles

#### Scenario: Nothing unmineable
- **WHEN** any generated tile kind is neither passable terrain nor bedrock
- **THEN** it has a mining drop

### Requirement: Deterministic feature stage
Features placed after terrain (deposits, natural stairs, creature spawns, and later ore spouts and reactors) SHALL each use their own generator seeded from the world seed, the chunk and the feature, so adding a feature never changes another's placement.

#### Scenario: Adding a feature
- **WHEN** a new feature is added to the stage
- **THEN** the deposits and natural stairs of every chunk stay the same

### Requirement: Old saves still load
A save from before this change SHALL load with every chunk it had already explored kept as it was. Chunks explored afterwards SHALL use the new generator.

#### Scenario: Old save
- **WHEN** a save from before this change is loaded
- **THEN** its explored chunks are unchanged, and newly explored chunks follow the depth bands
