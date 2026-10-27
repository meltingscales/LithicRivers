# factory-time Specification

## Purpose

How factory systems (machines, belts, generators, growth) advance with the game's turn-based time, where they simulate, and how the player lets time pass.

## Requirements

### Requirement: Factory steps follow elapsed game time
Factory systems SHALL advance in fixed steps of game ticks. Each player action SHALL run as many steps as fit in the ticks it took, carrying the remainder to the next action, so a factory's progress depends only on elapsed game time, not on how many actions it was split into. 100 ticks SHALL equal 1 game second.

#### Scenario: Same time, different actions
- **WHEN** one game passes 1000 ticks in a single action and another passes the same 1000 ticks as five 200-tick moves
- **THEN** every factory block in both games is in the same state

### Requirement: Simulate wherever placed
Every factory block SHALL keep simulating wherever it is in the world, whether or not the player is nearby.

#### Scenario: Factory away from the player
- **WHEN** the player walks two chunks away from a running machine and passes time
- **THEN** the machine progresses exactly as it would if the player were standing next to it

### Requirement: Pass time
A pass-time keybind SHALL advance the game by 10 game seconds, running factory steps, creature AI and all other systems as usual. Passing time SHALL stop early, at the tick it happened, if combat starts.

#### Scenario: Let the factory run
- **WHEN** the player presses the pass-time key once
- **THEN** the game tick advances by 1000 and factory blocks advance by the matching number of steps

#### Scenario: Interrupted by combat
- **WHEN** a feral dog reaches the player partway through passing time
- **THEN** passing time stops, combat starts, and the combat panel is shown

### Requirement: Factories run at a third of game speed
Factory block rates and power figures SHALL be measured in factory seconds, and one factory second SHALL last 3 game seconds (300 ticks), so a block works through one smelt or action in several player steps. A factory second SHALL be 5 factory steps.

#### Scenario: Smelting takes several steps
- **WHEN** a fueled crude furnace holds a log and the player walks six tiles (200 ticks each)
- **THEN** the log has been smelted into biochar, and not before the 12th game second
