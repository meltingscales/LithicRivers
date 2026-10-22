# Spec Delta

## ADDED Requirements

### Requirement: Planters and harvesters handle crops
The auto-planter SHALL plant any seed in its inventory, and the auto-harvester SHALL harvest ripe crops as well as grown trees.

#### Scenario: Wheat farm
- **WHEN** an auto-planter holds wheat seeds next to an auto-harvester on open grass
- **THEN** wheat is planted, ripens and is harvested into the harvester's output
