# Spec Delta

## Purpose

Store power on the grid in batteries and carry it in power cells, charged at batteries or by the robot's fusion core, to run gear.

## ADDED Requirements

### Requirement: Batteries store the grid's surplus
A battery block SHALL charge from its network's surplus and discharge into the network when demand exceeds supply.

#### Scenario: Riding out a fuel gap
- **WHEN** a charged battery is on a network whose generator runs out of fuel
- **THEN** its machines keep running from the battery until it's empty

### Requirement: Power cells in three tiers
Crude, standard and advanced power cells SHALL each hold a charge up to their tier's capacity, SHALL charge in a battery's cell slot, and SHALL show their charge in the inventory.

#### Scenario: Charging a cell
- **WHEN** an empty standard cell is placed in a charged battery's cell slot
- **THEN** its charge rises until it's full or the battery is empty

### Requirement: The fusion core charges carried cells
The robot's fusion core SHALL slowly charge the first carried cell that isn't full, faster when the core is in better shape.

#### Scenario: Trickle charge
- **WHEN** the robot carries an empty crude cell for one game minute with a working fusion core
- **THEN** the cell holds a charge

### Requirement: Gear draws from cells
Gear that needs power SHALL draw it from a carried cell when used, and SHALL refuse with a message when no cell has enough charge.

#### Scenario: Out of charge
- **WHEN** the player uses the biome compass with only empty cells
- **THEN** nothing happens and the message says there is no charge
