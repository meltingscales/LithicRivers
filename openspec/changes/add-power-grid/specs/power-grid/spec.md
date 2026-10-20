# Spec Delta

## Purpose

Supplies energy to machines: burner generators turn fuel into power, power poles connect generators and machines into networks, and networks share power between their consumers.

## ADDED Requirements

### Requirement: Burner generators burn only what is used
A burner generator SHALL accept fuel items (log, wooden plank, stick, wood shavings), each holding a fixed amount of energy, and SHALL produce power up to its maximum output. It SHALL consume fuel energy only equal to the power its network actually draws from it; when nothing draws power, it SHALL consume no fuel. Energy left in a partly used fuel item SHALL be kept for later.

#### Scenario: Idle grid
- **WHEN** a generator with 5 logs is connected to a network whose machines are all idle, and 10 game minutes pass
- **THEN** it still has 5 logs and no energy has been used

#### Scenario: Partial load
- **WHEN** a network draws half the generator's maximum output for 1 game minute
- **THEN** the generator uses half as much fuel energy as it would at full output, and any remainder of the current log stays available

#### Scenario: Non-fuel item
- **WHEN** the player tries to put stone into a generator
- **THEN** it is refused with a message that stone isn't fuel

### Requirement: Power poles form networks
A power pole SHALL connect every generator and power consumer within its supply area (a square of radius 3 around it) to its network, and SHALL link to every other pole within its wire reach (8 tiles). Poles linked directly or through other poles SHALL form one network. Removing a pole SHALL split a network if it was the only link.

#### Scenario: Two linked poles
- **WHEN** a generator is next to pole A, a machine is next to pole B, and A and B are 7 tiles apart
- **THEN** the machine is powered by the generator

#### Scenario: Out of reach
- **WHEN** the poles are 9 tiles apart
- **THEN** they are separate networks and the machine is unpowered

#### Scenario: Across a chunk edge
- **WHEN** pole A is at x = 60 and pole B at x = 66, in different chunks
- **THEN** they link exactly as two poles within one chunk

### Requirement: Sharing power
Within a network, the total demand of working consumers SHALL be met from all its generators, and generators SHALL share the load in proportion to their available output. When demand exceeds what the generators can supply, every consumer SHALL receive the same fraction of its demand and SHALL work at that fraction of its normal speed.

#### Scenario: Not enough power
- **WHEN** a network's consumers demand twice what its generators can supply
- **THEN** each consumer works at half speed

#### Scenario: Out of fuel
- **WHEN** every generator in a network runs out of fuel
- **THEN** the consumers stop and resume when fuel is added

### Requirement: Power lines on screen blink
Wires between linked poles SHALL be drawn as lines over the terrain between the poles, and SHALL blink: visible for one second, hidden for one second, repeating, so the terrain under them can be seen. Blinking SHALL follow the client clock, so headless runs draw the same frames every time.

#### Scenario: Blink
- **WHEN** two linked poles are on screen and the client clock advances one second
- **THEN** the wire between them switches between drawn and hidden

#### Scenario: Headless determinism
- **WHEN** the same headless script waits and dumps the screen twice in separate runs
- **THEN** both runs show the wire in the same state at the same step

### Requirement: Inspecting the grid
Interacting with a generator or pole SHALL show its network's current supply and demand, and, for a generator, its fuel items and the energy left in the current item.

#### Scenario: Inspect a generator
- **WHEN** the player interacts with a burning generator
- **THEN** a window shows the network's supply and demand and the generator's fuel

### Requirement: Generators and poles are craftable
Burner generators and power poles SHALL have recipes whose trees end in raw materials.

#### Scenario: Craft a pole
- **WHEN** the player has the ingredients and crafts power poles
- **THEN** power pole items are added to the inventory
