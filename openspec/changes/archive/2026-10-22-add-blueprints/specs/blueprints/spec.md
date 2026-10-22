# Spec Delta

## Purpose

Copy a player-built area and paste it elsewhere, rotated, paid for from the inventory.

## ADDED Requirements

### Requirement: Copying an area
In blueprint mode, the player SHALL be able to mark two corners of a rectangle and save the factory blocks and placed blocks inside as a blueprint, keeping each block's offset, kind, facing and filter, and leaving out contents and terrain.

#### Scenario: Copy a belt line
- **WHEN** the player copies an area holding three east-facing belts and a chest
- **THEN** the new blueprint holds three east belts and a chest at the same offsets, and no items

### Requirement: Blinking preview and rotation
While pasting, a blinking preview of the selected blueprint SHALL follow the cursor, SHALL rotate clockwise with the rotate key (turning every facing too), and SHALL mark tiles that would be replaced and tiles that block the paste.

#### Scenario: Rotate before pasting
- **WHEN** the player rotates a blueprint of east-facing belts once and pastes it
- **THEN** the placed belts face south and run down from the cursor

### Requirement: Paying for a paste
A paste SHALL use one item per block from the player's inventory and SHALL place nothing, listing what is missing, if any item is short.

#### Scenario: Not enough belts
- **WHEN** the player pastes a blueprint of 4 belts with 2 wooden conveyors in the inventory
- **THEN** nothing is placed and the message says 2 more wooden conveyors are needed

### Requirement: Replacing player-built blocks
If a paste would cover player-built blocks, the game SHALL ask for confirmation first. Confirming SHALL break those blocks into the player's inventory before placing. Terrain such as grass and dirt SHALL be built over without asking, and solid terrain such as rock and trees SHALL block the paste.

#### Scenario: Paste over a chest
- **WHEN** a paste would cover a chest the player built, and the player confirms
- **THEN** the chest item and its contents go to the inventory (or drop if it is full), and the blueprint is placed

### Requirement: Blueprint library
Blueprints SHALL be saved with the world, and the player SHALL be able to name them and select one from a Blueprints panel.

#### Scenario: Survives a reload
- **WHEN** the player saves a blueprint, then saves and loads the world
- **THEN** the blueprint is still in the library
