# Spec Delta

## Purpose

The Ruined City is a surface biome of rubble, ash, cracked roads and broken skyscrapers, the remains of the world before the reactor.

## ADDED Requirements

### Requirement: Rare, large cities
Worldgen SHALL make some surface areas Ruined City, decided by its own noise from the world seed, so cities are large, uncommon, and the same for the same seed. Outside cities the surface SHALL generate as it did before.

#### Scenario: Cities exist
- **WHEN** a large area of the surface is surveyed for a fixed seed
- **THEN** some of it is Ruined City and most of it is not

#### Scenario: Elsewhere unchanged
- **WHEN** a surface tile lies outside every city
- **THEN** it is the same tile it was before this change

### Requirement: City ground
City ground SHALL be ash and cracked road, which can be walked on, and rubble, which blocks movement and can be mined into stone and sometimes iron scrap. Cities SHALL have no grass or trees.

#### Scenario: Mining rubble
- **WHEN** the player mines rubble
- **THEN** they get stone, sometimes iron scrap, and the tile becomes ash

### Requirement: Skyscraper ruins
City chunks SHALL sometimes hold a skyscraper ruin: several floors joined by stairs, with walls of concrete, collapsed floors, scrap and loot, and sometimes a military android.

#### Scenario: A skyscraper
- **WHEN** many city chunks are generated for a fixed seed
- **THEN** some hold a skyscraper with more than one floor, reachable by stairs
