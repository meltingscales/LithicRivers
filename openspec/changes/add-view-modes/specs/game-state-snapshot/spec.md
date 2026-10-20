# Spec Delta

## MODIFIED Requirements

### Requirement: Snapshot contents
The snapshot SHALL include: format version, seed, current tick, player position, player body part states and integrity, inventory items with quantities, active UI tab and modal, the current view mode, whether combat is active and its participants, entities within the configured radius of the player (kind, position, health where applicable), and the most recent messages from the message log.

#### Scenario: Player and inventory present
- **WHEN** a new game with seed 12345 has started and the script runs `state`
- **THEN** the JSON contains seed 12345, the player's position, and the starting inventory items with their quantities

#### Scenario: Nearby entity listed
- **WHEN** an entity is within the snapshot radius of the player
- **THEN** it appears in the nearby-entities list with its kind and position

#### Scenario: View mode reported
- **WHEN** the player has switched to Power view and the script runs `state`
- **THEN** the JSON reports the view mode as Power
