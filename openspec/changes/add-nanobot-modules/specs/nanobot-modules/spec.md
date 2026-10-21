# Spec Delta

## Purpose

Nanobot modules in the robot's body build blueprints over time, faster with more and better modules.

## ADDED Requirements

### Requirement: Installing modules
Nanobot modules SHALL come in Mk1, Mk2 and Mk3 tiers with their own recipes, and SHALL be installed in up to 4 torso slots from the Body tab. A damaged or missing torso SHALL disable them.

#### Scenario: Install a module
- **WHEN** the player installs a Mk1 module in an empty torso slot
- **THEN** the module leaves the inventory and the Body tab shows it installed

### Requirement: Nanobots build blueprints
A blueprint paste SHALL need at least one working nanobot module, and SHALL place its blocks one at a time at a speed set by the total power of the installed modules, only within a range set by the highest tier, drawing energy from the robot's power source for each block.

#### Scenario: More modules build faster
- **WHEN** the same 20-block blueprint is pasted with one Mk1 module and then with four
- **THEN** four modules finish it in a quarter of the time

#### Scenario: Out of range
- **WHEN** queued blocks are farther from the player than the nanobot range
- **THEN** they wait, and are built once the player comes within range

### Requirement: Free nanobot cheat
The `nanobots_free` cheat SHALL toggle building without modules, placing pastes at once. It SHALL be refused in worlds without cheats.

#### Scenario: Cheat on
- **WHEN** cheats are allowed, the player runs `nanobots_free`, and pastes a blueprint with no modules installed
- **THEN** the whole blueprint is placed at once
