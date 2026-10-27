# belt-logistics Specification

## Purpose
Move items between blocks, sort and split belt lines, and let players walk across belts.

## Requirements

### Requirement: Extractor
An extractor SHALL move one item every factory second from the block behind it (a chest, a machine's output or input, or a generator's fuel) into the block in front of it, only when the block in front accepts the item. Items SHALL never be lost or duplicated.

#### Scenario: Chest to belt
- **WHEN** an extractor sits between a chest holding 3 logs and a belt
- **THEN** within 3 factory seconds the 3 logs are on the belt line and the chest is empty

#### Scenario: Blocked output
- **WHEN** the block in front of an extractor refuses items
- **THEN** the source keeps its items

### Requirement: Filter extractor
A filter extractor SHALL move only the item kind set as its filter, and nothing while no filter is set.

#### Scenario: Acorns back to the planter
- **WHEN** a filter extractor set to Acorn takes from a chest of logs and acorns into an auto-planter
- **THEN** only acorns reach the planter

### Requirement: Belt router
A belt router SHALL send items arriving from behind to its left, straight and right outputs in turn, skipping outputs that refuse the item. With a filter set, items of that kind SHALL go to the filter's side and others straight on.

#### Scenario: Even split
- **WHEN** 6 items arrive at a router with belts on all three outputs
- **THEN** each output belt receives 2

### Requirement: Walking on belts
The player SHALL be able to walk onto belts, and a belt SHALL carry a player standing on it one tile in its facing direction every factory step, faster than items ride, when the next tile is walkable and free. Other factory blocks SHALL stay solid.

#### Scenario: Ride a belt
- **WHEN** the player stands on the first of three east-facing belts and waits for two factory steps
- **THEN** the player is two tiles further east

### Requirement: Logistics blocks in the showcase
Each new block SHALL be craftable and SHALL appear in a Factory showcase example.

#### Scenario: Coverage
- **WHEN** the showcase coverage test runs
- **THEN** it passes with the new blocks included
