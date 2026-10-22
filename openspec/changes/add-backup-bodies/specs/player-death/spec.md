# Spec Delta

## ADDED Requirements

### Requirement: Respawning goes through the Resurrect screen
When backup bodies exist, the system-failure screen's respawn choice SHALL open the Resurrect screen instead of respawning at the bunker with the default body.

#### Scenario: Respawn choice
- **WHEN** the robot dies and the player chooses to respawn
- **THEN** the Resurrect screen opens, listing ready vats and the junker fallback
