# Spec Delta

## Purpose

Defines what a saved game contains and guarantees that loading a save restores exactly the state that was saved, so play continues as if the game had never been closed.

## ADDED Requirements

### Requirement: Loading restores the exact saved state
Loading a save SHALL restore every entity with all of its components and all game state resources exactly as they were when saved. The global config and the system scheduler are rebuilt at startup and are not part of a save.

#### Scenario: Damaged body survives a reload
- **WHEN** the player's right leg is damaged and the left arm is missing, and the game is saved and loaded
- **THEN** the right leg has the same integrity and state, the left arm is still missing, and walk speed is unchanged

#### Scenario: Corpse with loot survives a reload
- **WHEN** a feral dog has been killed and its corpse still holds leather and meat, and the game is saved and loaded
- **THEN** the corpse is at the same position with the same items, and can be looted

#### Scenario: Creatures keep their state
- **WHEN** a sheep has been hurt and a feral dog carries loot, and the game is saved and loaded
- **THEN** the sheep has the same health and the dog has the same inventory, health and AI state

#### Scenario: Toggles and log survive a reload
- **WHEN** noclip is on, fog of war is off, and the message log has entries, and the game is saved and loaded
- **THEN** noclip is still on, fog of war is still off, and the message log shows the same entries

### Requirement: Play continues identically after a reload
Saving and loading at any point SHALL not change what happens next: from the reloaded game, the same inputs SHALL produce the same states and screens as from the game that was never saved.

#### Scenario: Reload mid-combat
- **WHEN** the player is in combat with queued actions and the game is saved and loaded
- **THEN** combat is still active, the combat panel is shown, the queued actions are intact, and the next ticks resolve exactly as they would have without the reload

#### Scenario: Reload anywhere in a script
- **WHEN** a headless script is run once as written and once with `save-reload` inserted between two steps
- **THEN** both runs print identical output

### Requirement: Entity references stay valid
Entity identities SHALL be preserved across save and load, so anything that refers to another entity (combat targets, action queues, interaction targets) still points to the same entity.

#### Scenario: Combat target after reload
- **WHEN** the player has an attack queued against a specific dog and the game is saved and loaded
- **THEN** the queued attack still targets that dog

### Requirement: Nothing can be left out silently
Every component type that can appear on an entity SHALL be saveable. Adding a component type that is not registered for saving SHALL fail the test suite.

#### Scenario: Unregistered component
- **WHEN** a developer adds a new component type to entities without registering it for saving
- **THEN** `cargo test` fails and names the component type

### Requirement: Client play state is saved
Client state that affects play SHALL be saved with the game: tutorial progress, hotbar assignments, and the view position. Transient UI (open modals, menu selection, cheat console input) SHALL NOT be saved.

#### Scenario: Tutorial progress
- **WHEN** the player has completed three tutorial steps, saves and loads
- **THEN** the tutorial resumes at the fourth step

#### Scenario: Hotbar
- **WHEN** the player has assigned blocks to hotbar slots, saves and loads
- **THEN** the same blocks are in the same slots

### Requirement: Older saves still load
Saves in the previous format (version 4) SHALL still load using the previous rules, with a log message that the save was upgraded; the next save writes the current format.

#### Scenario: Version 4 save
- **WHEN** a version 4 save file is loaded
- **THEN** the game loads exactly as it did before this change, the log says the save was upgraded, and saving again writes version 5
