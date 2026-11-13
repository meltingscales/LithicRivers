# Spec Delta

## Purpose

"The Foundry", the fifth quest: SC-M4X7 sends the player to the SapienCorp foundry that cast the aperture's ring, and the patterns found there reveal the game's goal.

## ADDED Requirements

### Requirement: One foundry per world
Every world SHALL have the SapienCorp foundry at a fixed place on the surface, far from the bunker, with a quest marker shown once the quest starts.

#### Scenario: Always there
- **WHEN** a new world is generated with any seed
- **THEN** the foundry is at its fixed place, with its casting pit and pattern vault

### Requirement: Offered after quest 4
The repaired android SHALL offer "The Foundry" once "Under the River of Fire" is complete, and not before.

#### Scenario: Too early
- **WHEN** the player talks to the android before completing "Under the River of Fire"
- **THEN** "The Foundry" is not offered

#### Scenario: Offered
- **WHEN** the player talks to the android after "Under the River of Fire" is complete
- **THEN** the android offers "The Foundry", and accepting it starts the quest and shows the foundry's marker

### Requirement: The casting pit
The pattern vault SHALL lie across a molten casting pit that can only be crossed by bridging the lava with blocks.

#### Scenario: Bridging
- **WHEN** the player places plain blocks on the pit's lava in a line and walks across
- **THEN** they reach the vault side and the quest's "Cross the casting pit" step completes

### Requirement: The quest's steps
"The Foundry" SHALL complete its steps in order: reach the foundry, cross the casting pit, take the aperture casting patterns, and hand them to the android. Steps done early SHALL still count.

#### Scenario: Hand-in
- **WHEN** the player carrying the casting patterns talks to the android with the quest active
- **THEN** the patterns are handed over, the quest completes, and the Aperture is revealed

### Requirement: The goal is revealed
Handing in the patterns SHALL reveal the Aperture, and the android's dialogue SHALL tell the player that the frames were made to open it again and that the player is one of them.

#### Scenario: The reveal
- **WHEN** the patterns are handed in
- **THEN** the Aperture's recipes appear in the Crafting tab and its items in the Item Dictionary, and the android says the player was made to open it
