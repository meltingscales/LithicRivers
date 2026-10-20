# Spec Delta

## Purpose

The game's front door: a Minecraft-style title screen that leads to the world list, world creation and options, before any world is loaded.

## ADDED Requirements

### Requirement: Title screen
After the logo and game-title splash screens, the client SHALL show a title screen with the game logo, one splash-text line, and the choices **Play Game**, **Options** and **Quit**. No world is loaded or simulated while the title screen is shown.

#### Scenario: Reaching the title screen
- **WHEN** the game starts and the logo and game-title splashes finish (or are skipped with a key)
- **THEN** the title screen shows the logo, a splash-text line, and Play Game, Options and Quit, with Play Game selected

#### Scenario: Quit
- **WHEN** the player selects Quit
- **THEN** the game exits

### Requirement: Splash text
The splash-text line SHALL be one entry from a bundled list of short phrases, drawn in yellow beside or under the logo. A headless run SHALL always show the same entry.

#### Scenario: Deterministic in headless runs
- **WHEN** two headless runs reach the title screen
- **THEN** both show the same splash text

### Requirement: Menu navigation
Every menu screen SHALL be navigated with configurable keybinds (up, down, select, back), never hard-coded keys. Back SHALL return to the previous screen; on the title screen it does nothing.

#### Scenario: Back from the world list
- **WHEN** the player is on the world list and presses the back keybind
- **THEN** the title screen is shown

### Requirement: Options screen
The Options screen SHALL let the player change music volume and sound-effect volume in steps, apply the change immediately, and save it to the config file so it persists across launches.

#### Scenario: Volume persists
- **WHEN** the player lowers the music volume, leaves Options, and restarts the game
- **THEN** the music plays at the lowered volume and Options shows the saved value
