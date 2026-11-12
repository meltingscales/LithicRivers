# Spec Delta

## Purpose

The scrap workbench breaks crafted items back into some of their ingredients, on grid power.

## ADDED Requirements

### Requirement: Uncrafting with a recovery chance
A powered scrap workbench SHALL break a full batch of an item (the quantity its recipe makes) into that recipe's ingredients, recovering each ingredient unit with a 50% chance. The rolls SHALL be reproducible from the world seed, the workbench's position and the number of jobs it has done.

#### Scenario: About half comes back
- **WHEN** a workbench scraps many batches of an item whose recipe uses 4 iron scrap
- **THEN** it recovers about half of the iron scrap those batches cost, never more than the cost of any one batch per batch

#### Scenario: Full batches only
- **WHEN** a workbench holds 2 wooden conveyors and their recipe makes 2
- **THEN** it scraps them as one batch; with only 1 conveyor it waits

#### Scenario: Same world, same scrap
- **WHEN** two worlds with the same seed scrap the same items in a workbench at the same place
- **THEN** they recover the same items

### Requirement: What can be scrapped
A scrap workbench SHALL accept only items made by exactly one recipe. Raw materials and items made by more than one recipe SHALL be refused, from the window and from belts.

#### Scenario: Raw material
- **WHEN** the player tries to put a log into a workbench
- **THEN** it is refused and stays in the inventory

#### Scenario: Two recipes
- **WHEN** wooden shavings, which two recipes make, arrive on a belt
- **THEN** the workbench doesn't take them

### Requirement: Power and timing
A scrap workbench SHALL take 4 factory seconds per batch at full power, run slower when its network is short, and pause without power, keeping its progress.

#### Scenario: Unpowered
- **WHEN** a workbench holds a batch and has no power
- **THEN** it scraps nothing until power returns, then finishes the batch

### Requirement: Belt lines
A scrap workbench SHALL take items from the block behind it and from its sides, and push recovered items out of its front into the next block or onto a belt.

#### Scenario: A scrapping line
- **WHEN** a belt feeds pipe guns into a powered workbench's back and a chest sits at its front
- **THEN** the chest fills with recovered iron scrap, planks and nails

### Requirement: Craftable early
The scrap workbench SHALL be craftable from early materials and placed like other factory blocks.

#### Scenario: Reachable
- **WHEN** the crafting reachability check runs
- **THEN** the scrap workbench is reachable from raw sources
