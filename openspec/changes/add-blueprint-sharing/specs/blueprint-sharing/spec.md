# Spec Delta

## Purpose

Share blueprints between worlds and players as text, and keep large libraries organized.

## ADDED Requirements

### Requirement: Blueprint strings
A blueprint or book SHALL export as one line starting with `LITHICRIVERS-BLUEPRINT-`, and importing that line SHALL add an identical blueprint or book to the library.

#### Scenario: Round trip
- **WHEN** the player exports a blueprint and imports its string into another world
- **THEN** the imported blueprint has the same name, size, blocks, facings and filters

#### Scenario: Bad string
- **WHEN** the player pastes text without the prefix, a damaged payload, or a newer format version
- **THEN** nothing is added and the message says why

### Requirement: Export reaches the player
Exporting SHALL show the string in the Blueprints window, try to copy it to the system clipboard, and save it to a text file.

#### Scenario: Export
- **WHEN** the player exports a blueprint named "Farm"
- **THEN** the window shows its string and a file named after it holds the same string

### Requirement: Blueprint books
The library SHALL hold books of blueprints, nested if the player wants. Blueprints and books SHALL be movable between books, renamable and deletable.

#### Scenario: Move into a book
- **WHEN** the player creates a book named "Power" and moves a blueprint into it
- **THEN** the tree shows the blueprint under "Power"

### Requirement: Removing kinds of blocks
The player SHALL be able to remove every block of one kind from a blueprint.

#### Scenario: Remove the chests
- **WHEN** the player removes "Chest" from a blueprint holding 3 belts and 2 chests
- **THEN** the blueprint holds the 3 belts and no chests
