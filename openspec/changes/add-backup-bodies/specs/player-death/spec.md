# Spec Delta

## ADDED Requirements

### Requirement: Respawning goes through the Resurrect screen
When backup bodies exist, the system-failure screen's respawn choice SHALL open the Resurrect screen instead of respawning at the bunker with the default body.

#### Scenario: Respawn choice
- **WHEN** the robot dies and the player chooses to respawn
- **THEN** the Resurrect screen opens, listing ready vats and the junker fallback

## MODIFIED Requirements

### Requirement: Respawning
Respawning SHALL wake the player in the chosen vat's spare body beside that vat, or in a junker body at the SapienCorp HQ (at the respawn point in worlds without the HQ), with the starting kit, and SHALL leave the world, factories and quests as they were.

#### Scenario: Respawn
- **WHEN** the player chooses the junker body with no vat built
- **THEN** the player stands outside the SapienCorp HQ, alive, with every part damaged and the starting kit, and the wreck is still where they died
