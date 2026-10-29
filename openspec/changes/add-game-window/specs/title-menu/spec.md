# Spec Delta

## MODIFIED Requirements

### Requirement: Options screen
The Options screen SHALL let the player change music volume and sound-effect volume in steps, change the window's text size in steps, and turn fullscreen on or off. Each change SHALL apply immediately and be saved to the config file so it persists across launches. In terminal mode, Text size and Fullscreen SHALL be shown as not available, since the terminal controls them.

#### Scenario: Volume persists
- **WHEN** the player lowers the music volume, leaves Options, and restarts the game
- **THEN** the music plays at the lowered volume and Options shows the saved value

#### Scenario: Text size from Options
- **WHEN** the player raises Text size in Options while in the game window
- **THEN** the text gets bigger immediately, and the next launch uses that size

#### Scenario: Terminal mode options
- **WHEN** the player opens Options in terminal mode
- **THEN** Text size and Fullscreen say they aren't available in terminal mode
