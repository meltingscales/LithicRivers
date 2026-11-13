# Spec Delta

## Purpose

The Aperture is the game's goal: a biomechanical gate the player builds and keeps fed with endgame materials until it opens.

## ADDED Requirements

### Requirement: Hidden until revealed
Before the Aperture is revealed, its recipes SHALL NOT appear in the Crafting tab, its items SHALL NOT appear in the Item Dictionary or the hotbar picker, and its blocks SHALL NOT be placeable. After the reveal, all of them SHALL. The reveal SHALL be saved with the world.

#### Scenario: Before and after
- **WHEN** the Crafting tab and Item Dictionary are searched for the Aperture before and after the reveal
- **THEN** nothing is found before, and its core, frame and port are found after, also after a save and reload

### Requirement: The multiblock
The Aperture SHALL be an 11-by-11 square with a core at its center, frame blocks around its border, and one input port for each input kind at set border positions. It SHALL work only when complete, and SHALL say which positions are missing when it isn't.

#### Scenario: Incomplete
- **WHEN** one frame block of an otherwise complete Aperture is missing
- **THEN** it doesn't charge, and its window names the missing position

#### Scenario: Complete
- **WHEN** every core, frame and port block is in place
- **THEN** the Aperture is complete and starts tracking its inputs

### Requirement: Inputs and their rates
Each input port SHALL take only its item kind (perfect biomech embryos, meat slurry, ethanol, mash or biochar) from belts or extractors, and SHALL track how many arrived in each of the last ten game minutes. Every input SHALL be something the factory can make without end.

#### Scenario: Wrong item
- **WHEN** a belt carries biochar into the ethanol port
- **THEN** the port doesn't take it

### Requirement: Charge
Each game minute, a complete Aperture SHALL judge every input over its last ten minutes (or fewer, early on), gain one point when every input met its required rate, gain the lowest input's fraction of a point when any fell short, and lose a point when any input was below half its rate for three minutes in a row. At 120 points it SHALL open. Judging over ten minutes lets slow inputs, like a fraction of an embryo a minute, count fairly.

#### Scenario: Full rates
- **WHEN** every input meets its rate for 120 game minutes
- **THEN** the Aperture opens

#### Scenario: A short input
- **WHEN** one input runs at 60% of its rate for a minute and the rest meet theirs
- **THEN** the charge rises by 0.6 of a point that minute

#### Scenario: A long shortage
- **WHEN** one input stays below half its rate for three minutes
- **THEN** the charge falls by a point

### Requirement: The flow window
The Aperture's window SHALL list each input with its required rate, its actual rate over the last minute and the last ten minutes, and a color: green at 110% or more and not falling, yellow from 95% to 110%, red below 95% or falling. It SHALL show the charge and an estimate of the time left at the current rates.

#### Scenario: Colors
- **WHEN** one input runs at 120%, one at 100% and one at 80%
- **THEN** they show green, yellow and red

### Requirement: The ending
When the Aperture opens, the game SHALL play its closing sequence and the credits, then return to the world with the Aperture standing open. Stepping into the open Aperture SHALL say the far side isn't ready yet. Opening SHALL be saved.

#### Scenario: Opening
- **WHEN** the charge reaches 120 points
- **THEN** the closing sequence and credits play, and afterwards the player is in the world beside the open Aperture

### Requirement: Endgame scale stays fast
A large running factory feeding the Aperture SHALL keep ticks and frames within the performance budget.

#### Scenario: Bench
- **WHEN** the endgame bench scenario runs
- **THEN** its ticks and frames stay inside the 16 ms budget
