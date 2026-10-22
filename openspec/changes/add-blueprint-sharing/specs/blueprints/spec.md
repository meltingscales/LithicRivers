# Spec Delta

## MODIFIED Requirements

### Requirement: Blueprint library
Blueprints and blueprint books SHALL be saved with the world, and the player SHALL be able to name them, organize them in books, and select one from a Blueprints panel that shows the library as a tree.

#### Scenario: Survives a reload
- **WHEN** the player saves a blueprint, then saves and loads the world
- **THEN** the blueprint is still in the library

#### Scenario: Old libraries load
- **WHEN** a save made before books is loaded
- **THEN** its blueprints are at the top level of the library
