# Spec Delta

## MODIFIED Requirements

### Requirement: Input steps
`key <category>.<ACTION> [count]` SHALL press the first key bound to that keybind, `count` times (default 1). `rawkey <NAME>` SHALL press a key by its config key name. `type "<text>"` SHALL press one character key per character. Each press SHALL be handled exactly as the interactive client handles that key. `pad <button> [count]` SHALL tap a gamepad button. `pad hold <button>` SHALL press and keep holding one, past the wheel delay, and `pad release <button>` SHALL let it go. `pad <stick> <direction>` SHALL push a stick or D-pad one step in a direction (`n`, `ne`, `e`, `se`, `s`, `sw`, `w`, `nw`). All of them go through the same mapping the game window uses.

#### Scenario: Keybind press
- **WHEN** a script runs `key movement.MOVE_NORTH 3`
- **THEN** the client handles three presses of the key bound to `movement:MOVE_NORTH`, as if a player pressed it three times

#### Scenario: Unbound keybind
- **WHEN** a script runs `key movement.FLY`
- **THEN** the run fails with status 1 and an error naming the line and the unknown keybind

#### Scenario: Gamepad button
- **WHEN** a script runs `pad a` beside a chest on the World page
- **THEN** the chest opens, as when a player presses A

#### Scenario: Holding for a wheel
- **WHEN** a script runs `pad hold lb`, `pad rstick n` and `pad release lb` on the World page
- **THEN** the wheel's north action happens, as when a player does the same

#### Scenario: Unknown pad input
- **WHEN** a script runs `pad z`
- **THEN** the run fails with status 1 and an error naming the line and the unknown button
