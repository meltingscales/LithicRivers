# Spec Delta

## Purpose

The game's goal: a hidden biomechanical teleporter that opens only after a factory has fed it endgame items at a steady rate for a long time.

## ADDED Requirements

### Requirement: The teleporter multiblock
An 11-by-11 frame of teleporter blocks around a core, with input ports on its border, SHALL form the teleporter, and an incomplete frame SHALL show which positions are missing.

#### Scenario: Incomplete
- **WHEN** the frame is missing two blocks
- **THEN** the teleporter's window marks the two missing positions and doesn't charge

### Requirement: Charging by sustained input
The teleporter SHALL charge while every input arrives at its required rate, charge slower when an input falls short, lose charge during a long shortage, and open when fully charged.

#### Scenario: A shortage
- **WHEN** one input runs at half its rate for 3 game minutes
- **THEN** the charge goes down

### Requirement: The item-flow window
The teleporter's window SHALL show each input's required rate, its recent actual rate, and a green, yellow or red status, with the charge and an estimate of the time left.

#### Scenario: Falling input
- **WHEN** an input's rate is below 95% of its requirement
- **THEN** it shows red

### Requirement: Hidden until revealed
Before its reveal, the teleporter's recipes, blocks and Item Dictionary entries SHALL not appear anywhere in the game.

#### Scenario: Before the reveal
- **WHEN** a player who hasn't reached the reveal opens the crafting tab and the Item Dictionary
- **THEN** neither lists anything of the teleporter

### Requirement: Large factories stay fast
A factory step with 2,000 factory blocks SHALL stay within the tick budget in release builds.

#### Scenario: Benchmark
- **WHEN** the large-factory benchmark runs in a release build
- **THEN** one factory step takes less than its budget
