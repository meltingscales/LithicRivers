# Spec Delta

## Purpose

Molten ore spouts are permanent sources of liquid ore, mostly underground and less often on the surface. This spec covers how they appear in the world, how they fill and stop, and how their ore becomes ingots.

## ADDED Requirements

### Requirement: Spouts appear underground and on the surface
Worldgen SHALL place molten ore spouts in underground layers and on the surface layer. A surface chunk SHALL be half as likely to get a spout as an underground chunk. Where spouts appear SHALL depend only on the world seed and chunk coordinates, so the same seed always has spouts in the same places. Spouts SHALL be rare enough that finding one is a goal.

#### Scenario: Fewer on the surface
- **WHEN** spouts are counted over the same large number of surface chunks and underground chunks for a fixed seed
- **THEN** the surface has about half as many spouts per chunk as underground

#### Scenario: Same seed, same spouts
- **WHEN** two worlds are created with the same seed and the same underground chunk is generated in both
- **THEN** both chunks have spouts at the same positions with the same ore

### Requirement: Spouts fill up to a cap
A spout SHALL add its molten ore to its own reservoir at a fixed rate per factory step, until the reservoir reaches its capacity, and then stop. Draining the reservoir SHALL let it fill again. Spouts SHALL fill wherever they are, like every factory block.

#### Scenario: Fill and stop
- **WHEN** an untouched spout has passed enough game time to fill completely
- **THEN** its reservoir is at capacity and passing more time adds nothing

#### Scenario: Refill after draining
- **WHEN** 300 mB are pumped out of a full spout and time passes
- **THEN** the spout fills back up to capacity at its fixed rate

### Requirement: Spouts can't be broken
Mining a spout SHALL leave it in place and say that it can't be broken.

#### Scenario: Mine a spout
- **WHEN** the player mines a molten iron spout
- **THEN** the spout and its reservoir are unchanged and the message log says it can't be broken

### Requirement: Scooping with a crucible
Interacting with a spout while carrying an empty crucible SHALL move 100 mB of its ore into the crucible, if the spout has at least that much. Interacting with a casting basin while carrying a full crucible SHALL pour it into the basin if the basin has room.

#### Scenario: Scoop and pour
- **WHEN** the player scoops from a spout holding 500 mB of molten iron and pours into an empty casting basin
- **THEN** the spout holds 400 mB, the basin holds 100 mB, and the crucible is empty again

### Requirement: Casting ingots
A casting basin SHALL turn each 100 mB of molten ore into one ingot of that metal after a fixed cooling time. It SHALL push finished ingots into the block it faces when that block accepts items, such as a belt or a chest, and otherwise keep them.

#### Scenario: Cast into a chest
- **WHEN** a casting basin facing a chest holds 200 mB of molten iron and the cooling time passes twice
- **THEN** the chest holds 2 iron ingots and the basin is empty
