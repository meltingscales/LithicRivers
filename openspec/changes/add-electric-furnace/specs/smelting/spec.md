# Spec Delta

## ADDED Requirements

### Requirement: Electric furnace
An electric furnace SHALL smelt using the shared smelting table, SHALL have no fuel slot, and SHALL draw power from its network only while it is smelting. It SHALL smelt at a speed proportional to its network's power satisfaction and SHALL not smelt when unpowered.

#### Scenario: Powered smelting
- **WHEN** a fully powered electric furnace has a log in its input and the log's smelting time passes
- **THEN** 1 biochar is in its output

#### Scenario: Idle draws nothing
- **WHEN** an electric furnace has no input
- **THEN** it demands no power

#### Scenario: Half power
- **WHEN** its network can supply half the demand
- **THEN** each smelt takes twice as long

### Requirement: Electric furnace routing
An electric furnace SHALL take items arriving from its back or either side into its input, and SHALL push finished items out its front.

#### Scenario: Side input
- **WHEN** a belt of logs enters an electric furnace from the side
- **THEN** the logs go into its input and are smelted

### Requirement: Electric furnaces are craftable
Electric furnaces SHALL have a recipe whose tree ends in raw materials.

#### Scenario: Craft an electric furnace
- **WHEN** the player has the ingredients and crafts an electric furnace
- **THEN** an electric furnace item is added to the inventory
