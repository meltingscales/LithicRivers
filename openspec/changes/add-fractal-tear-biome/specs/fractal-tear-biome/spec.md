# Spec Delta

## Purpose

An endgame biome of fractal-shaped caverns in the deep, holding the raw materials of the last factory tier.

## ADDED Requirements

### Requirement: Rare tears in the deep
Fractal Tears SHALL generate as rare regions in the deep band, never within 2,000 tiles of the world's origin, and SHALL be the same for the same seed.

#### Scenario: Not near spawn
- **WHEN** the area within 2,000 tiles of the origin is generated
- **THEN** it holds no Fractal Tear blocks

### Requirement: Fractal shapes
A tear's caverns SHALL follow fractal functions (escape-time sets, Menger-sponge lattices and bifurcation seams), with tear glass on set boundaries.

#### Scenario: Deterministic shape
- **WHEN** the same tear region is generated twice
- **THEN** every tile is the same

### Requirement: Tear materials
Tears SHALL hold petrogen shale, uranic crust and fractal residue, each solid and mineable with its own drop, and tear glass, which can't be mined and doesn't block light.

#### Scenario: Mining a tear
- **WHEN** the player mines petrogen shale
- **THEN** they get petrogen shale items
