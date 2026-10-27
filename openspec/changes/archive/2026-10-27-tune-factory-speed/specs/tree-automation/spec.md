# Spec Delta

## MODIFIED Requirements

### Requirement: Auto-planter
An auto-planter SHALL hold up to 16 acorns in its input, accept acorns from a belt facing into it or from the player, and, while powered, plant one acorn every 4 factory seconds at full power (drawing 60 kW) on a free grass or dirt tile within a square of radius 2 around it, choosing tiles in a fixed order. It SHALL do nothing when it has no acorns or no free tile.

#### Scenario: Planting around itself
- **WHEN** a powered auto-planter with 3 acorns is surrounded by free grass and 12 factory seconds pass
- **THEN** 3 saplings have been planted within 2 tiles of it in the fixed order, and it has no acorns left

#### Scenario: Fed by a belt
- **WHEN** a belt carrying acorns faces into an auto-planter
- **THEN** the acorns move from the belt into the planter's input

### Requirement: Auto-harvester
An auto-harvester SHALL, while powered, chop one grown tree every 6 factory seconds at full power (drawing 180 kW, more than one burner generator gives) within a square of radius 2 around it, choosing trees in a fixed order. The drops SHALL go into its output buffer (up to 16 items), and the harvester SHALL push items from its output into the tile it faces: a belt, a chest or a machine that accepts them. When its output is full, it SHALL stop chopping.

#### Scenario: Harvest onto a belt
- **WHEN** a powered auto-harvester faces east into a belt and a grown tree is within 2 tiles
- **THEN** within 6 factory seconds the tree becomes dirt and its logs and acorns start moving along the belt

#### Scenario: Full output
- **WHEN** the harvester's output is full and nothing accepts items in front of it
- **THEN** it stops chopping until there is space
