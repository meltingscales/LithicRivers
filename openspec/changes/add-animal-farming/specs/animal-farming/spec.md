# Spec Delta

## Purpose

Animal farms as factories: breed with feed, automate feeding and counting, cull with a salvaged processor, and grow graded biomechanical embryos from the meat.

## ADDED Requirements

### Requirement: Breeding with feed
Two fed sheep near each other SHALL produce a lamb that grows into a sheep, then wait a cooldown, and no breeding SHALL happen when too many sheep are nearby.

#### Scenario: Lamb
- **WHEN** the player feeds carrots to two sheep standing next to each other
- **THEN** a lamb appears near them, and it grows into a sheep after 5 game minutes

### Requirement: Auto-feeders and counters
An auto-feeder SHALL feed unfed animals near it from its stock on a timer, and an animal counter SHALL show how many animals of each kind are in its square.

#### Scenario: Counting
- **WHEN** an animal counter set to 5x5 has 3 sheep in its square
- **THEN** it shows 3 sheep

### Requirement: The livestock processor
A powered livestock processor SHALL damage non-player creatures in front of it, never the player, and collect their drops into its inventory.

#### Scenario: Processing
- **WHEN** a sheep stands in front of a powered processor
- **THEN** it takes damage until it dies, and its meat ends up in the processor's inventory

### Requirement: Bio-vats grow graded embryos
A powered bio-vat SHALL turn meat, mash and scrap electronics into biomechanical embryos graded deformed, wrinkly or perfect by a seeded roll, with more meat improving the odds.

#### Scenario: Grades
- **WHEN** a bio-vat runs 100 batches
- **THEN** most embryos are deformed, some wrinkly, a few perfect, and the same world gives the same results
