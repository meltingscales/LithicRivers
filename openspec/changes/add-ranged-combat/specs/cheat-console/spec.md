# Spec Delta

## ADDED Requirements

### Requirement: Spawn a military android
`spawn_enemy military_android [x y z]` SHALL spawn a rusty military android with full health and its seeded ammo.

#### Scenario: Spawn
- **WHEN** the player runs `spawn_enemy military_android 5 0 0`
- **THEN** a military android with full health and ammo stands at (5, 0, 0)
