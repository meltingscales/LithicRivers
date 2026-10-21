# Spec Delta

## Purpose

The thermoelectric generator turns a radioactive item into small, endless power, the first power source that needs no fuel.

## ADDED Requirements

### Requirement: Heat source
A thermoelectric generator SHALL hold at most one heat source: nuclear waste or a uranium chunk. The player SHALL be able to put one in and take it out through the generator's window. The generator SHALL never consume its heat source.

#### Scenario: Load a heat source
- **WHEN** the player puts a uranium chunk into an empty generator
- **THEN** the generator holds it, and passing any amount of time leaves it there

### Requirement: Constant power
A loaded thermoelectric generator SHALL supply a fixed amount of power to its network every factory step for as long as it is loaded: a lower amount with nuclear waste and a higher amount with a uranium chunk. An empty generator SHALL supply nothing.

#### Scenario: Waste versus uranium
- **WHEN** one generator holds nuclear waste and another holds a uranium chunk
- **THEN** the uranium generator supplies about twice the power of the waste generator, and both keep supplying it indefinitely

#### Scenario: Empty
- **WHEN** a generator has no heat source
- **THEN** it supplies no power

### Requirement: Craftable after ore spouts
The thermoelectric generator recipe SHALL need iron ingots, scrap electronics and reactor casing scrap, so it can only be made after the player can cast iron and has visited a ruin.

#### Scenario: Recipe ingredients
- **WHEN** the crafting reachability test runs
- **THEN** the thermoelectric generator is craftable from raw sources, and its ingredient tree includes iron ingots and reactor casing scrap
