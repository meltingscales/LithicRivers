# Spec Delta

## Purpose

The sulfur slime is a yellow slime of the deep's lava rivers that always carries sulfur, giving gunpowder a source that can be farmed.

## ADDED Requirements

### Requirement: Lives by the lava rivers
Sulfur slimes SHALL appear only in the deep band, and only within a few tiles of lava.

#### Scenario: Not in the shallows
- **WHEN** natural spawning runs in the shallows for a long time
- **THEN** no sulfur slime appears there

### Requirement: Slow chase, like a nuclear slime
A sulfur slime SHALL notice the player within 6 tiles on its level and move one tile toward them every other turn. Combat SHALL start when it is adjacent.

#### Scenario: Closes in
- **WHEN** a sulfur slime is 4 tiles east of the player and the player waits
- **THEN** after 6 turns it is adjacent and combat has started

### Requirement: Combat stats
A sulfur slime SHALL have less health than a nuclear slime and more than a feral dog. Its attack rolls SHALL come from the world's seeded random number generator.

#### Scenario: Between a dog and a nuclear slime
- **WHEN** a feral dog, a sulfur slime and a nuclear slime are spawned
- **THEN** the sulfur slime's health is above the dog's and below the nuclear slime's

### Requirement: Always carries sulfur
Every sulfur slime SHALL carry 1 to 3 sulfur, rolled when it spawns from the world seed and its spawn position, and SHALL leave it in a corpse when it dies.

#### Scenario: Killing one
- **WHEN** a sulfur slime is killed
- **THEN** a corpse remains holding 1 to 3 sulfur

#### Scenario: Same slime, same sulfur
- **WHEN** two worlds with the same seed spawn a sulfur slime at the same position
- **THEN** both carry the same amount of sulfur
