# Spec Delta

## ADDED Requirements

### Requirement: Mining bots in the deep, away from the bunker
Mining bots SHALL spawn naturally only in the deep band, and never within 128 tiles (horizontally) of the SapienCorp bunker.

#### Scenario: Deep only
- **WHEN** the spawn rules run around a player in the shallows or on the surface
- **THEN** no mining bot spawns

#### Scenario: Not by the bunker
- **WHEN** the spawn rules run for many checks around a player in the starting bunker
- **THEN** no mining bot spawns there

#### Scenario: Far from the bunker
- **WHEN** the spawn rules run for many checks around a player in the deep band far from the bunker
- **THEN** mining bots spawn there
