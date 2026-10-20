# headless-client Specification

## Purpose

Lets AI agents and CI run the full game client without a terminal by executing a deterministic script and reading the resulting screens and state as text.

## Requirements

### Requirement: Headless script execution
The client SHALL accept a script file and execute it without a terminal, raw mode, alternate screen, or audio device, then exit. `just client-blind <script>` SHALL invoke this mode.

#### Scenario: Run a script without a terminal
- **WHEN** `just client-blind scripts/agent/smoke.txt` runs in a non-interactive shell
- **THEN** the client executes every step, prints its dumps to stdout, and exits without modifying the terminal state

#### Scenario: No audio device available
- **WHEN** a script runs on a machine with no audio output device
- **THEN** the run completes normally and no sound is played

### Requirement: Script syntax
A script SHALL be UTF-8 text with one step per line. Blank lines and lines starting with `#` SHALL be ignored. Each step SHALL be a command word followed by its arguments.

#### Scenario: Comments and blank lines
- **WHEN** a script contains blank lines and `# note` lines between steps
- **THEN** those lines are skipped and the remaining steps run in order

#### Scenario: Unknown command
- **WHEN** a script contains `jump 3`
- **THEN** the run stops before executing any step, prints an error naming line number and command, and exits with status 1

### Requirement: Run setup steps
`seed <u64>` and `size <cols>x<rows>` SHALL only appear before the first non-setup step and SHALL set the world seed and screen size. Without them the run SHALL use the configured `DEFAULT_SEED` and a 160x50 screen. `skip-splash` SHALL jump straight to the main game UI.

#### Scenario: Seed after gameplay step
- **WHEN** `seed 5` appears after a `key` step
- **THEN** validation fails with status 1 and an error naming that line

#### Scenario: Skip splash
- **WHEN** a script starts with `skip-splash` followed by `screen`
- **THEN** the dumped screen shows the main game UI, not the logo or boot message

### Requirement: Input steps
`key <category>.<ACTION> [count]` SHALL press the first key bound to that keybind, `count` times (default 1). `rawkey <NAME>` SHALL press a key by its config key name. `type "<text>"` SHALL press one character key per character. Each press SHALL be handled exactly as the interactive client handles that key.

#### Scenario: Keybind press
- **WHEN** a script runs `key movement.MOVE_NORTH 3`
- **THEN** the client handles three presses of the key bound to `movement:MOVE_NORTH`, as if a player pressed it three times

#### Scenario: Unbound keybind
- **WHEN** a script runs `key movement.FLY`
- **THEN** the run fails with status 1 and an error naming the line and the unknown keybind

### Requirement: Time and tick steps
`wait <ms>` SHALL advance the client's clock by that many milliseconds in frame-sized increments, running per-frame updates (splash timers, combat auto-ticks) as the interactive loop would. `tick [n]` SHALL advance the game simulation by `n` ticks (default 1) and SHALL apply each tick's combat transitions to the client the same way a player action does: combat starting opens the combat panel on the World tab, and combat ending closes it. No step SHALL depend on real elapsed time.

#### Scenario: Combat auto-tick under fake time
- **WHEN** combat is active, `COMBAT_MS_PER_TICK` is 1000, and the script runs `wait 3000`
- **THEN** the game advances exactly three combat ticks, however long the run takes in real time

#### Scenario: Combat starts during a tick step
- **WHEN** a feral dog reaches the player during a `tick` step
- **THEN** the state dump reports combat active and the screen shows the combat panel, as it would after a player action that took the same tick

### Requirement: Cheat steps
`cheat <command line>` SHALL execute the line through the in-game cheat console command registry and SHALL fail the run with status 1 if the command is unknown or returns an error.

#### Scenario: Teleport via cheat
- **WHEN** a script runs `cheat tp 10 10 0` then `state`
- **THEN** the state dump reports the player at (10, 10, 0)

### Requirement: Screen dumps
`screen [label]` SHALL render the full client UI at the run's screen size and print it as plain text between `=== screen <label> ===` and `=== end screen ===` lines, one text row per screen row, with trailing spaces trimmed.

#### Scenario: Screen dump framing
- **WHEN** a script with `size 120x40` runs `screen start`
- **THEN** stdout contains `=== screen start ===`, at most 40 text lines no wider than 120 columns, and `=== end screen ===`

### Requirement: Expectations
`expect-text "<text>"` SHALL check that the current rendered screen contains the text. On failure the run SHALL print the line number, the expected text and the current screen, then exit with status 2.

#### Scenario: Expectation fails
- **WHEN** the screen does not contain `You chop the tree` and the script runs `expect-text "You chop the tree"`
- **THEN** the run prints the failing line and the current screen, and exits with status 2

### Requirement: Exit status
The run SHALL exit 0 when every step succeeds, 1 on a script, setup, or command error, and 2 on a failed expectation. A `quit` step SHALL end the run successfully.

#### Scenario: Successful run
- **WHEN** every step in a script succeeds
- **THEN** the process exits with status 0

### Requirement: Deterministic output
Two runs of the same script with the same seed, screen size, and game build SHALL produce byte-identical stdout.

#### Scenario: Repeat run
- **WHEN** the same script is run twice
- **THEN** both runs print identical output

### Requirement: Interactive client unchanged
Running the client without a script SHALL behave as before: real time, terminal UI, audio.

#### Scenario: Normal launch
- **WHEN** a player runs `just client-linux`
- **THEN** splash timing, combat auto-ticks, music, and input behave as they did before this change

### Requirement: Save-reload step
`save-reload` SHALL save the running game (including client play state) to a temporary location and load it back into the same session, then continue the script. It SHALL fail the run with status 1 if saving or loading fails.

#### Scenario: Transparent reload
- **WHEN** a script runs `key movement.MOVE_SOUTH`, `save-reload`, `key movement.MOVE_SOUTH`, `state`
- **THEN** the state dump is identical to the same script without the `save-reload` line

#### Scenario: Reload through MCP
- **WHEN** an MCP client runs `save-reload` via the `run_script` tool
- **THEN** the session continues from the reloaded state and the step is included in `export_script`
