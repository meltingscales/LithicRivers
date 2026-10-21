# Spec Delta

## Purpose

The nuclear slime is a slow, sturdy underground enemy that sometimes carries nuclear waste, giving the nuclear tier a second source besides ruined reactors.

## ADDED Requirements

### Requirement: Slow chase
A nuclear slime SHALL notice the player within 6 tiles on its level, then move one tile toward the player every other turn. Combat SHALL start when it is adjacent, the same way as with a feral dog.

#### Scenario: Slime closes in
- **WHEN** a nuclear slime is 4 tiles east of the player and the player waits
- **THEN** after 6 turns the slime is adjacent and combat has started

### Requirement: Combat stats
A nuclear slime SHALL have more health than a feral dog and SHALL attack less often but for more damage per hit. Its attack rolls SHALL come from the world's seeded random number generator, so a fight replays identically.

#### Scenario: Tougher than a dog
- **WHEN** a nuclear slime and a feral dog are spawned
- **THEN** the slime has more health than the dog

### Requirement: Rare nuclear waste drop
Each nuclear slime SHALL have a 5% chance to carry one nuclear waste, rolled when it spawns from the world seed and its spawn position. When a slime that carries waste dies, it SHALL leave a corpse holding the waste; a slime without waste SHALL leave nothing.

#### Scenario: Drop rate
- **WHEN** 2000 slimes are spawned at different positions in one world
- **THEN** between 3% and 7% of them carry nuclear waste

#### Scenario: Same slime, same drop
- **WHEN** two worlds with the same seed spawn a slime at the same position
- **THEN** both slimes carry the same items

#### Scenario: Killing a slime with waste
- **WHEN** a slime that carries nuclear waste is killed
- **THEN** a corpse remains on its tile and looting it gives one nuclear waste

#### Scenario: Killing a slime without waste
- **WHEN** a slime that carries nothing is killed
- **THEN** it leaves no corpse

### Requirement: Underground spawning
Nuclear slimes SHALL spawn only in underground chunks, at a seeded chance per chunk, and never on the surface.

#### Scenario: Not on the surface
- **WHEN** a large area of surface chunks and a large area of underground chunks are generated for a fixed seed
- **THEN** no slimes are on the surface and at least one is underground

### Requirement: Nuclear waste is a raw source
Nuclear waste dropped by slimes SHALL count as a raw source for crafting, through the same creature loot table the drop uses.

#### Scenario: Raw sources
- **WHEN** the crafting raw-source list is built
- **THEN** it contains nuclear waste
