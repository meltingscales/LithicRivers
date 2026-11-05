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

### Requirement: Transit stations activate on first use
SapienCorp transit stations SHALL start dormant. Interacting with a dormant station SHALL activate it and say so, without opening the destination list. Dormant stations SHALL NOT appear in any destination list. Activation SHALL be saved with the world.

#### Scenario: Activating a station
- **WHEN** the player interacts with the dormant Factory Station for the first time
- **THEN** a message says the transit node is registered, the player doesn't move, and the Factory Station appears in other stations' destination lists from then on

### Requirement: Free travel between stations
Interacting with an activated station SHALL list every other activated station. Choosing one SHALL move the player to a free tile next to it at no energy cost.

#### Scenario: Station to station
- **WHEN** the player, with the Bunker and Factory Stations activated, uses the Bunker Station to travel to the Factory Station
- **THEN** the player stands next to the Factory Station

### Requirement: Mobile tele-access device
Using the mobile tele-access device SHALL list every player-built teleporter and every activated station. A jump SHALL cost the same as from a teleporter at the player's position, drawn from carried power cells. If the cells can't cover it, the jump SHALL not happen and a message SHALL say there is no charge.

#### Scenario: Device jump
- **WHEN** the player, carrying a charged power cell, uses the device to travel to the Bunker Station
- **THEN** the player stands next to the Bunker Station and the cell's charge has dropped by the jump cost

#### Scenario: Device with no charge
- **WHEN** the player uses the device with only empty power cells
- **THEN** the player doesn't move and is told there is no charge

### Requirement: A station where the player wakes up
The SapienCorp bunker SHALL have a transit station in the room where a new player wakes up.

#### Scenario: New world
- **WHEN** a new world starts
- **THEN** a dormant transit station stands in the player's starting room in the bunker
