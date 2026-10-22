# Spec Delta

## Purpose

Lua scripts on SapienNet computers for automation too complex for logic blocks, stored as plain files in the world's folder and run safely and deterministically.

## ADDED Requirements

### Requirement: Computers run scripts
A SapienNet computer SHALL run its Lua script once per factory step, reading the network's signals, and SHALL be able to emit signals and enable or disable connected blocks.

#### Scenario: Stop the feeder
- **WHEN** a computer's script disables the feeder when `sheep >= 12`, and the network reads 13 sheep
- **THEN** the feeder is disabled from that step on

### Requirement: Scripts are world files
Each computer's script SHALL be a `.lua` file in the world's `scripts/` folder, editable outside the game, and changes on disk SHALL be picked up while playing.

#### Scenario: External edit
- **WHEN** the player edits a computer's script in another editor and saves it
- **THEN** the computer runs the new script from the next step

### Requirement: Sandboxed and deterministic
Scripts SHALL have no access to files, the clock or system randomness, SHALL get a seeded `random()`, and SHALL stop with a shown error when they fail or exceed their instruction budget, without affecting the rest of the factory.

#### Scenario: An infinite loop
- **WHEN** a script loops forever
- **THEN** it stops with "ran out of time", and every other block keeps working

### Requirement: The in-game IDE
The game SHALL let the player edit a computer's script in a window with line numbers, the network's live signals, the last error with its line, and the script's printed output.

#### Scenario: An error
- **WHEN** a script has a syntax error on line 3
- **THEN** the IDE shows the error on line 3 and the computer stays stopped until it's fixed
