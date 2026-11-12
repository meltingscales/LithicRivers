# Spec Delta

## ADDED Requirements

### Requirement: Spawn a sulfur slime
`spawn_enemy sulfur_slime [x y z]` SHALL spawn a sulfur slime with full health and its seeded sulfur.

#### Scenario: Spawn
- **WHEN** the player runs `spawn_enemy sulfur_slime 5 0 0`
- **THEN** a sulfur slime with full health stands at (5, 0, 0), carrying sulfur
