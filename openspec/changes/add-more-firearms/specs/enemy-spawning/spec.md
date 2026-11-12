# Spec Delta

## ADDED Requirements

### Requirement: Sulfur slimes spawn by the lava
Natural spawning SHALL add sulfur slimes in the deep near lava, within the same per-area limits as other creatures, and sulfur slimes SHALL drop embryos of their own kind that grow sulfur slimes in a spawner.

#### Scenario: Natural spawn
- **WHEN** the player waits in the deep beside a lava river for several spawn checks
- **THEN** sulfur slimes can appear out of sight, never more than the area's cap

#### Scenario: Spawner farm
- **WHEN** a powered spawner is fed meat slurry and a sulfur slime embryo
- **THEN** it grows sulfur slimes
