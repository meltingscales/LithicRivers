# Spec Delta

## Purpose

Buildable teleporters that move the player between their own bases, paid for from battery banks charged by the power grid.

## ADDED Requirements

### Requirement: Battery bank charges from the grid
A teleporter SHALL store energy up to its capacity, drawing power from its network only while it isn't full, at a rate limited by its maximum charge rate and by the network's satisfaction.

#### Scenario: Charging
- **WHEN** an empty teleporter is on a network with spare power for 60 game seconds
- **THEN** its charge has grown by its charge rate times 60 seconds (or to its capacity)

### Requirement: Travelling between teleporters
Interacting with a teleporter SHALL list every other teleporter by name and distance. Choosing one SHALL move the player to a free tile next to it and spend the jump's cost from the source teleporter's charge. The cost SHALL grow with distance.

#### Scenario: A jump
- **WHEN** the player uses a charged teleporter to travel to "North Base"
- **THEN** the player stands next to the North Base teleporter and the source's charge has dropped by the jump cost

### Requirement: Not enough charge
If the source teleporter's charge is below the jump cost, the jump SHALL not happen, and a message SHALL say that the battery bank needs more charge and how much.

#### Scenario: Short on charge
- **WHEN** the player tries a jump costing 20 MJ from a teleporter holding 12 MJ
- **THEN** the player doesn't move and is told the battery bank needs 8 MJ more

### Requirement: Naming
New teleporters SHALL get a unique default name, and the player SHALL be able to rename them from the teleporter's window.

#### Scenario: Rename
- **WHEN** the player renames a teleporter to "Furnace Row"
- **THEN** other teleporters list it as "Furnace Row"
