# Spec Delta

## ADDED Requirements

### Requirement: The world's save file
A world's game state SHALL be saved as `world.json.gz` in its folder. Playing a world SHALL load `world.json.gz` if present, else the older `world.json`, and the first save afterwards SHALL replace the older file.

#### Scenario: Upgrading a world folder
- **WHEN** a world folder holds only `world.json` from before this change, and the player plays and saves it
- **THEN** the folder holds `world.json.gz`, no `world.json`, and the world plays as it was
