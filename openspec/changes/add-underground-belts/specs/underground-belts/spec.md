# Spec Delta

## Purpose

Let belt lines cross on one layer by carrying items under the tiles between two linked underground conveyors.

## ADDED Requirements

### Requirement: Linked pairs
An underground conveyor placed facing the same way as an unlinked entrance, ahead of it in line and at most 10 tiles away, SHALL become that entrance's exit. Otherwise it SHALL be an entrance.

#### Scenario: Link a pair
- **WHEN** the player places an east-facing underground conveyor, then another east-facing one 6 tiles east of it
- **THEN** the first is an entrance linked to the second, its exit

#### Scenario: Out of range
- **WHEN** the second one is 11 tiles away
- **THEN** both are unlinked entrances

### Requirement: Carrying items under other blocks
An entrance SHALL take items like a belt and deliver them, in order, out of its exit, taking as long as belts over the same distance, whatever lies between.

#### Scenario: Crossing a belt line
- **WHEN** a north-running belt line passes between a linked east-facing pair, and logs ride into the entrance
- **THEN** the logs come out of the exit and the north line keeps its own items

### Requirement: Breaking an end
Breaking either end SHALL give its item back, drop every item in transit on the entrance's tile, and leave the other end unlinked.

#### Scenario: Break mid-transit
- **WHEN** the exit is broken while three logs are in the tunnel
- **THEN** the three logs lie on the entrance's tile and nothing is lost
