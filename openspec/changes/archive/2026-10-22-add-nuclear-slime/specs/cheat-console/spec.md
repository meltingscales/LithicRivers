# Spec Delta

## MODIFIED Requirements

### Requirement: Spawn enemy
`spawn_enemy <kind> [x y z]` SHALL spawn one enemy of the given kind at the given position, with the same components, health and seeded inventory as an enemy spawned by the game itself. Supported kinds: `feral_dog`, `nuclear_slime`.

#### Scenario: Spawn a feral dog
- **WHEN** the player is at (0, 0, 0) and runs `spawn_enemy feral_dog 3 0 0`
- **THEN** a feral dog with full health stands at (3, 0, 0) and the console logs where it was spawned

#### Scenario: Spawn a nuclear slime
- **WHEN** the player is at (0, 0, -20) and runs `spawn_enemy nuclear_slime 3 0 -20`
- **THEN** a nuclear slime with full health stands at (3, 0, -20) and the console logs where it was spawned

#### Scenario: Unknown kind
- **WHEN** a player runs `spawn_enemy dragon`
- **THEN** nothing is spawned and the command fails with a message listing the supported kinds
