# Spec Delta

## Purpose

Sound effects: which player actions make sounds, and how they are played without touching the simulation.

## ADDED Requirements

### Requirement: Actions make sounds
The game SHALL play a sound when the player chops a tree, mines a tile or breaks a block, places a block or machine, picks up items, damages an enemy, takes damage, teleports, or crafts a recipe. Machines mining SHALL NOT make the mining sound.

#### Scenario: Mining
- **WHEN** the player mines a rock
- **THEN** the mining sound is queued

#### Scenario: Chopping
- **WHEN** the player chops a tree
- **THEN** the chopping sound is queued

#### Scenario: Combat
- **WHEN** an enemy's attack damages the player
- **THEN** the hurt sound is queued

#### Scenario: Harvester
- **WHEN** an auto-harvester fells a tree
- **THEN** no sound is queued

### Requirement: Sounds don't change the game
Queued sounds SHALL NOT be saved and SHALL NOT affect the simulation, so the same script gives the same output with or without audio.

#### Scenario: Save and reload
- **WHEN** a world with queued sounds is saved and loaded
- **THEN** the save holds no sounds

### Requirement: Every sound has an asset
Every sound cue SHALL have an audio file bundled with the game.

#### Scenario: Assets
- **WHEN** the client starts
- **THEN** each cue's sound is found in the embedded assets
