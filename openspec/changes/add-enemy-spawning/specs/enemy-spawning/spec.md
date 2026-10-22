# Spec Delta

## Purpose

Creatures keep appearing where they belong, within limits, and players can build farms around spawner blocks with fences.

## ADDED Requirements

### Requirement: Areas refill within limits
Creatures SHALL keep spawning by place-based rules, never in view, never within 16 tiles of the player, and never beyond each kind's cap within its radius.

#### Scenario: A cleared forest
- **WHEN** the player kills every dog near a forest and waits there out of view of it
- **THEN** dogs reappear over time, never more than the cap within its radius

#### Scenario: Never in view
- **WHEN** a spawn rule fires
- **THEN** no creature appears on a tile the player can currently see

### Requirement: Bases are safe from natural spawns
Natural spawns SHALL not happen near placed light sources or inside areas enclosed by fences and walls.

#### Scenario: Fenced base
- **WHEN** the player fences in an area
- **THEN** no creature spawns inside it

### Requirement: Spawner blocks
A spawner block SHALL spawn its creature nearby on a timer while the player is in range, up to its own cap, and breaking it SHALL give a spawner core that can be placed again.

#### Scenario: Farm
- **WHEN** a dog spawner sits inside a fenced pen and the player waits nearby
- **THEN** dogs appear in the pen, never more than four from that spawner

### Requirement: Fences block creatures
Creatures SHALL not pass through fences, and fences SHALL not block sight or light.

#### Scenario: Behind a fence
- **WHEN** a dog chases the player and a fence is between them
- **THEN** the dog can't reach the player through the fence, and the player still sees it
