# Spec Delta

## Purpose

An attract mode that shows short, real cuts of LithicRivers gameplay from the title screen.

## ADDED Requirements

### Requirement: Watch Demo
The title screen SHALL offer a Watch Demo choice that starts the demo.

#### Scenario: Start the demo
- **WHEN** the player selects Watch Demo on the title screen
- **THEN** the first demo scenario starts playing with the DEMO banner showing

### Requirement: Demo scenarios
The demo SHALL play exploring, NPC dialogue, combat, building and factory scenarios in a fixed order and loop, each a real game world driven by a script and played back 1.5 times slower than its script's timings, each ending after its script or 30 seconds, with a cut between them. Playback SHALL be deterministic: the same build always shows the same frames.

#### Scenario: Cut to the next scenario
- **WHEN** a scenario's script ends
- **THEN** the demo cuts to the next scenario, and after the last one it starts again from the first

#### Scenario: Same every time
- **WHEN** the demo is run twice headlessly for the same number of frames
- **THEN** both runs print identical screens

### Requirement: DEMO banner
While the demo plays, the top row SHALL show a red banner with "DEMO" repeated and scrolling horizontally over time, and the current scenario's name.

#### Scenario: Banner scrolls
- **WHEN** 100 ms of the clock pass during the demo
- **THEN** the banner text has shifted by one column

### Requirement: Leaving the demo
Escape SHALL end the demo and return to the title screen; other keys SHALL do nothing. The demo SHALL never write to the saves folder or the config.

#### Scenario: Escape
- **WHEN** the player presses Escape during the demo
- **THEN** the title screen shows with Watch Demo selected, and no world was saved

### Requirement: Scenarios stay working
Every demo scenario SHALL run as a test, so a change that breaks one fails `cargo test`.

#### Scenario: A broken scenario
- **WHEN** a change renames a keybind that a demo script uses
- **THEN** `cargo test` fails naming the scenario
