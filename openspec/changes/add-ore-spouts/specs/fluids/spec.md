# Spec Delta

## Purpose

Defines how factory blocks hold liquids, how pumps move them, and how pipes carry them to machines for processing.

## ADDED Requirements

### Requirement: Fluid storage
A block that holds fluid SHALL hold a single fluid kind at a time, as an amount in millibuckets with a fixed capacity. An empty block SHALL accept any fluid; a block with fluid SHALL only accept more of the same kind.

#### Scenario: Different fluid refused
- **WHEN** a pump tries to push molten iron into a basin that already holds molten copper
- **THEN** nothing moves

### Requirement: Pumps move fluid
A powered pump SHALL move fluid each factory step from the block behind it into the block in front of it, at most its rate per step, limited by what the source holds and the target has room for. An unpowered pump SHALL move nothing. Pumps SHALL work across chunk edges like every factory block.

#### Scenario: Pump from a spout
- **WHEN** a powered pump faces away from a full spout toward an empty basin, and one factory step runs
- **THEN** the spout loses exactly the pump's rate and the basin gains it

#### Scenario: No power
- **WHEN** the same pump is unpowered
- **THEN** nothing moves

### Requirement: Pipe networks
Pipes SHALL connect to adjacent pipes in the four cardinal directions, and each connected group SHALL form one network. A network SHALL hold a single fluid kind, with room for 100 mB per pipe. A pump facing into any pipe of a network SHALL push into that network. Networks SHALL span chunk edges like every factory block.

#### Scenario: Pump through pipes
- **WHEN** a powered pump next to a spout pushes into a line of 5 pipes that ends at a casting basin
- **THEN** molten iron reaches the basin and casting begins

#### Scenario: Network across a chunk edge
- **WHEN** a pipe line crosses from x = 63 to x = 64
- **THEN** it is one network and delivers fluid exactly as a line inside one chunk would

### Requirement: Changing a pipe network
Placing or breaking a pipe SHALL rebuild the affected networks. Breaking a pipe SHALL lose that pipe's share of the fluid, and the rest SHALL stay in the networks that remain.

#### Scenario: Break a pipe in the middle
- **WHEN** a line of 5 pipes holding 500 mB of molten iron has its middle pipe broken
- **THEN** there are two networks of 2 pipes holding 400 mB together, and the message log says 100 mB was lost

### Requirement: Machines take fluid from pipes
A machine with a fluid input SHALL accept fluid from any adjacent pipe network, on any side except its front. Each factory step a network SHALL deliver to its adjacent machines in a fixed order (by position), giving each as much as it has room for, up to the network's throughput per step. A machine SHALL refuse a fluid it can't process.

#### Scenario: Two basins on one network
- **WHEN** one pipe network with 300 mB of molten iron touches two empty basins
- **THEN** the basins fill in a fixed order that is the same in every run, and together receive at most the network's throughput per step

#### Scenario: Wrong fluid
- **WHEN** a network of molten copper touches a basin that holds molten iron
- **THEN** nothing is delivered to that basin

### Requirement: Fluid state is saved
Fluid amounts and kinds, including what pipe networks hold, SHALL be saved and loaded with the world like every other component.

#### Scenario: Reload a half-full basin
- **WHEN** a basin holding 60 mB of molten iron is saved and loaded
- **THEN** it holds 60 mB of molten iron
