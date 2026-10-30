# title-menu Specification

## Purpose

The game's front door: a Minecraft-style title screen that leads to the world list, world creation and options, before any world is loaded.

## Requirements

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
The Options screen SHALL let the player change music volume and sound-effect volume in steps, change the game window's text size in steps, turn fullscreen on or off, and choose whether the Display panel is docked, floating or hidden. Each change SHALL apply immediately and be saved to the config file so it persists across launches. In terminal mode, Text size, Fullscreen and Display SHALL be shown as not available, since the terminal controls them.

#### Scenario: Volume persists
- **WHEN** the player lowers the music volume, leaves Options, and restarts the game
- **THEN** the music plays at the lowered volume and Options shows the saved value

#### Scenario: Text size from Options
- **WHEN** the player raises Text size in Options while in the game window
- **THEN** the text gets bigger immediately, and the next launch uses that size

#### Scenario: Terminal mode options
- **WHEN** the player opens Options in terminal mode
- **THEN** Text size, Fullscreen and Display say they aren't available in terminal mode
