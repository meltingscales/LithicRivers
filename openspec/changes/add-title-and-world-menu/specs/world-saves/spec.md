# Spec Delta

## Purpose

Lets players keep several named worlds, each with its own seed and settings, and choose which one to play, create or delete from the title menu.

## ADDED Requirements

### Requirement: World list
**Play Game** SHALL open a list of saved worlds found in the saves folder, most recently played first, each showing its name, seed and last-played date, followed by **Create New World**. Selecting a world SHALL offer **Play** and **Delete**.

#### Scenario: No worlds yet
- **WHEN** the saves folder has no worlds and the player selects Play Game
- **THEN** the list shows only Create New World

#### Scenario: Order
- **WHEN** world "A" was last played after world "B"
- **THEN** "A" is listed above "B"

#### Scenario: Unreadable world
- **WHEN** a world folder's files are missing or corrupt
- **THEN** that world is listed as unreadable, can be deleted, cannot be played, and the other worlds still load

### Requirement: Create a world
**Create New World** SHALL let the player set a world name (default "New World"), a seed, **Allow cheats** (default off), **Developer mode** (default from the config's `DEVELOPER_MODE`) and **Show tutorial** (default on), then create the world, play the boot-message intro, and start the game.

#### Scenario: Numeric seed
- **WHEN** the player enters seed `12345`
- **THEN** the world uses seed 12345 and its starting area matches any other world with seed 12345

#### Scenario: Text seed
- **WHEN** the player enters seed `lithic`
- **THEN** the seed is a fixed hash of the text, so entering `lithic` again gives the same world

#### Scenario: Blank seed
- **WHEN** the seed field is left blank
- **THEN** the world gets a new seed, shown in the world list afterwards

#### Scenario: Duplicate name
- **WHEN** a world named "New World" already exists and the player creates another with that name
- **THEN** both worlds exist as separate folders and both appear in the list

### Requirement: Save and load a world
Each world SHALL be stored in its own folder under the saves folder. Saving SHALL write the world's game state, settings and last-played time to that folder; playing a world SHALL restore the state it was last saved in.

#### Scenario: Round trip
- **WHEN** the player moves, picks up items, saves, quits to the title screen, and plays the same world again
- **THEN** the player's position, inventory, the tick and the world's settings are as they were when saved

#### Scenario: Save and quit to title
- **WHEN** the player chooses Save and Quit to Title from the in-game Menu tab
- **THEN** the world is saved and the title screen is shown

#### Scenario: Quit tab
- **WHEN** the player activates the in-game Quit tab, which reads "SAVE AND QUIT"
- **THEN** the world is saved and the title screen is shown; quitting the game is done from the title screen

### Requirement: Delete a world
Deleting a world SHALL require a confirmation and then remove that world's folder.

#### Scenario: Confirmed delete
- **WHEN** the player selects Delete on a world and confirms
- **THEN** the world disappears from the list and its folder is removed

#### Scenario: Cancelled delete
- **WHEN** the player selects Delete and then cancels
- **THEN** the world is unchanged

### Requirement: Per-world settings apply in game
A loaded world's settings SHALL control the game: developer mode and tutorial visibility come from the world, not the global config, and cheats follow the world's Allow cheats setting.

#### Scenario: Tutorial off
- **WHEN** a world is created with Show tutorial off
- **THEN** the game starts without the Tutorial tab
