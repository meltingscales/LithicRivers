# Spec Delta

## ADDED Requirements

### Requirement: Cheats follow the world setting
The in-game cheat console SHALL be available only in worlds that allow cheats. In a world without cheats, the cheat-console keybind SHALL not open the console and SHALL show a message that cheats are disabled for this world. Headless `cheat` steps and the MCP `cheat` tool are test tooling and SHALL run in any world.

#### Scenario: Cheats disabled
- **WHEN** a world was created with Allow cheats off and the player presses the cheat-console keybind
- **THEN** the console does not open and the message log says cheats are disabled for this world

#### Scenario: Cheats enabled
- **WHEN** a world allows cheats and the player presses the cheat-console keybind
- **THEN** the console opens as before
