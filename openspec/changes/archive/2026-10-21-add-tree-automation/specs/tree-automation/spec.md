# Spec Delta

## Purpose

Machines that run a tree farm without the player: an auto-planter plants acorns around itself and an auto-harvester chops grown trees around itself and outputs the wood.

## ADDED Requirements

### Requirement: Auto-planter
An auto-planter SHALL hold up to 16 acorns in its input, accept acorns from a belt facing into it or from the player, and, while powered, plant one acorn every 2 game seconds on a free grass or dirt tile within a square of radius 2 around it, choosing tiles in a fixed order. It SHALL do nothing when it has no acorns or no free tile.

#### Scenario: Planting around itself
- **WHEN** a powered auto-planter with 3 acorns is surrounded by free grass and 6 game seconds pass
- **THEN** 3 saplings have been planted within 2 tiles of it in the fixed order, and it has no acorns left

#### Scenario: Fed by a belt
- **WHEN** a belt carrying acorns faces into an auto-planter
- **THEN** the acorns move from the belt into the planter's input

### Requirement: Auto-harvester
An auto-harvester SHALL, while powered, chop one grown tree every 3 game seconds within a square of radius 2 around it, choosing trees in a fixed order. The drops SHALL go into its output buffer (up to 16 items), and the harvester SHALL push items from its output into the tile it faces: a belt, a chest or a machine that accepts them. When its output is full, it SHALL stop chopping.

#### Scenario: Harvest onto a belt
- **WHEN** a powered auto-harvester faces east into a belt and a grown tree is within 2 tiles
- **THEN** within 3 game seconds the tree becomes dirt and its logs and acorns start moving along the belt

#### Scenario: Full output
- **WHEN** the harvester's output is full and nothing accepts items in front of it
- **THEN** it stops chopping until there is space

### Requirement: Power use
Each machine SHALL demand power only while it has work to do (acorns and a free tile, or a grown tree and output space), and SHALL work at the speed its network's power satisfaction allows. Unpowered machines SHALL not work.

#### Scenario: Idle machine draws nothing
- **WHEN** an auto-harvester has no grown trees in range
- **THEN** it demands no power and its generator burns no fuel for it

#### Scenario: Half power
- **WHEN** the network can supply half of the machines' demand
- **THEN** each machine takes twice as long per action

### Requirement: Machines across chunk edges
A machine's work area SHALL extend into neighboring chunks the same way it does within one chunk.

#### Scenario: Area spanning chunks
- **WHEN** an auto-harvester stands at x = 63, y = 63 with grown trees at x = 64..65 and y = 64..65
- **THEN** it harvests them as it would within one chunk

### Requirement: Machines are craftable
Auto-planters and auto-harvesters SHALL have recipes whose trees end in raw materials.

#### Scenario: Craft an auto-harvester
- **WHEN** the player has the ingredients and crafts an auto-harvester
- **THEN** an auto-harvester item is added to the inventory

### Requirement: Self-running farm
A farm of auto-planter, auto-harvester, belt into a chest, a burner generator and power poles SHALL keep producing wood with no player input once it has initial acorns and fuel.

#### Scenario: Reference farm
- **WHEN** the reference farm script passes 30 game minutes after setup
- **THEN** the chest holds more logs than were used as fuel, and the state is identical across runs
