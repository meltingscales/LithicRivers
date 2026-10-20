# Spec Delta

## Purpose

Turns wood-type materials into biochar, a denser fuel, using furnaces that fit into belt lines.

## ADDED Requirements

### Requirement: Smelting table
Smelting SHALL follow one table shared by every furnace: log → 1 biochar, 2 wooden planks → 1 biochar, 4 sticks → 1 biochar, 4 acorns → 1 biochar, 8 wood shavings → 1 biochar, each with its own smelting time. Items not in the table SHALL NOT be accepted as smelting input.

#### Scenario: Smelt a log
- **WHEN** a fueled crude furnace has a log in its input and its smelting time passes
- **THEN** the log is gone and 1 biochar is in the output

#### Scenario: Batch input
- **WHEN** a furnace has 3 wooden planks in its input and enough time passes for one smelt
- **THEN** 2 planks are consumed, 1 biochar is produced, and 1 plank waits for another plank

#### Scenario: Not smeltable
- **WHEN** the player tries to put stone into a furnace's input
- **THEN** it is refused with a message that stone can't be smelted

### Requirement: Crude furnace burns fuel only while smelting
A crude furnace SHALL accept fuel items in its fuel slot, SHALL burn fuel energy only while it is smelting, and SHALL keep the remaining energy of a partly burned fuel item. With no fuel it SHALL not smelt. It needs no power.

#### Scenario: Idle furnace
- **WHEN** a crude furnace has fuel but no input for 10 game minutes
- **THEN** its fuel is untouched

#### Scenario: Runs out of fuel
- **WHEN** a furnace runs out of fuel halfway through a smelt
- **THEN** smelting pauses and resumes from where it stopped when fuel is added

### Requirement: Biochar is fuel
Biochar SHALL be accepted as fuel by crude furnaces and burner generators and SHALL hold more energy than a log.

#### Scenario: Biochar in a generator
- **WHEN** biochar is put into a burner generator's fuel slot
- **THEN** it is accepted and provides more energy than a log

### Requirement: Furnaces fit into belt lines
A furnace SHALL take items arriving from its back into its input, items arriving from its left or right into its fuel slot, and SHALL push finished items out its front into a belt, chest or machine that accepts them. Items that don't fit the slot they arrive at SHALL stay on the belt.

#### Scenario: Belt-fed furnace
- **WHEN** a belt of logs enters a furnace from the back, a belt of biochar enters from the side, and a belt leads away from its front
- **THEN** logs are smelted, the furnace fuels itself from the side belt, and biochar leaves on the front belt

#### Scenario: Wrong side
- **WHEN** a belt carrying stone enters the furnace from the side
- **THEN** the stone stays on the belt because stone isn't fuel

### Requirement: Furnace window
Interacting with a furnace SHALL open a window showing the fuel, input and output slots, smelting progress and remaining fuel energy, where the player can move items in and out.

#### Scenario: Load a furnace by hand
- **WHEN** the player opens a furnace and moves 4 logs into the input and 2 logs into the fuel slot
- **THEN** the furnace starts smelting and the window shows the progress

### Requirement: Crude furnaces are craftable
Crude furnaces SHALL have a recipe whose tree ends in raw materials.

#### Scenario: Craft a crude furnace
- **WHEN** the player has the ingredients and crafts a crude furnace
- **THEN** a crude furnace item is added to the inventory
