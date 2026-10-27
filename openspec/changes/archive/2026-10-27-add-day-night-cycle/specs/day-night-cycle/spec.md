# Spec Delta

## Purpose

A 15-minute day that darkens the surface at night, shows the time, and makes feral dogs bolder in the dark.

## ADDED Requirements

### Requirement: Time of day
The time of day SHALL follow the world's tick: a day lasts 90,000 ticks (15 game minutes), tick 0 is 08:00, and night runs from 21:00 to 05:00, with dawn and dusk around it.

#### Scenario: A new world
- **WHEN** a new world starts
- **THEN** it is 08:00 and daytime

### Requirement: Night on the surface
At night, the surface SHALL use the same fog of war and light radius as underground. By day it SHALL be fully lit. Underground layers SHALL not change with the time of day.

#### Scenario: Midnight on the surface
- **WHEN** it is 00:00 and the player stands on the surface without a torch
- **THEN** only tiles within 2 tiles of the player are lit, and seen tiles beyond are remembered in grayscale

#### Scenario: Underground at noon
- **WHEN** it is 12:00 and the player is underground
- **THEN** lighting is the same as at any other time

### Requirement: Clock
The MODES panel SHALL show the time of day and whether it is day or night.

#### Scenario: Reading the clock
- **WHEN** it is 22:10
- **THEN** the MODES panel shows a night glyph and 22:10

### Requirement: Bolder dogs at night
Feral dogs SHALL notice the player from farther away at night than by day.

#### Scenario: Night hunt
- **WHEN** a feral dog is 20 tiles from the player at night
- **THEN** it chases the player, and at noon it would not
