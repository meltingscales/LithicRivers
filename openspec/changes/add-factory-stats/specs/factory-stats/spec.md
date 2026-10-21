# Spec Delta

## Purpose

Show how power and items flow over time, so players can tell whether their factory keeps up and where items go.

## ADDED Requirements

### Requirement: Power history
Each power network SHALL record its supply and demand once per game second for the last 10 game minutes, and the pole info box SHALL show it as a graph of the last 60 seconds or 10 minutes, marking seconds when demand exceeded supply.

#### Scenario: A short network shows in the graph
- **WHEN** a network's demand exceeds its supply for 5 game seconds
- **THEN** the pole info box's 60-second graph marks those 5 seconds as short

### Requirement: Item flow by source
The game SHALL count every item produced and consumed, by item and source (player mining, each machine kind, crafting, fuel burned, planting), per game minute for the last 60 minutes.

#### Scenario: Chopping by hand and by machine
- **WHEN** the player chops one tree and an auto-harvester chops another
- **THEN** the log counts show the player's logs under player mining and the harvester's under the auto-harvester

### Requirement: Production panel
A Production tab SHALL list items with their produced and consumed rates over a chosen range (1 minute, 10 minutes or 1 hour), and show the sources of the selected item.

#### Scenario: Rates for logs
- **WHEN** a tree farm has produced 30 logs and burned 3 in the last 10 minutes
- **THEN** the Production tab, set to 10 minutes, shows logs at 3 produced and 0.3 consumed per minute

### Requirement: Stats are saved
Power history and item flow SHALL be saved with the world, so saving and loading changes neither.

#### Scenario: Reload
- **WHEN** a world with recorded stats is saved and loaded
- **THEN** the graphs and the Production tab show the same values
