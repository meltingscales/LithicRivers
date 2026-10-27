# Spec Delta

## MODIFIED Requirements

### Requirement: Extractor
An extractor SHALL move one item every factory second from the block behind it (a chest, a machine's output or input, or a generator's fuel) into the block in front of it, only when the block in front accepts the item. Items SHALL never be lost or duplicated.

#### Scenario: Chest to belt
- **WHEN** an extractor sits between a chest holding 3 logs and a belt
- **THEN** within 3 factory seconds the 3 logs are on the belt line and the chest is empty

#### Scenario: Blocked output
- **WHEN** the block in front of an extractor refuses items
- **THEN** the source keeps its items

### Requirement: Walking on belts
The player SHALL be able to walk onto belts, and a belt SHALL carry a player standing on it one tile in its facing direction every factory step, faster than items ride, when the next tile is walkable and free. Other factory blocks SHALL stay solid.

#### Scenario: Ride a belt
- **WHEN** the player stands on the first of three east-facing belts and waits for two factory steps
- **THEN** the player is two tiles further east
