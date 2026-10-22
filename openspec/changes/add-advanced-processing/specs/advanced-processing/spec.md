# Spec Delta

## Purpose

Endgame processing chains that turn the Fractal Tear's raw materials into fuel, nuclear fuel and rare catalysts, with a use for every byproduct.

## ADDED Requirements

### Requirement: Petrogen distillation
Petrogen shale SHALL be crushed into slurry, distilled into tar, oil and gas, and the oil refined into petrogen fuel, the densest fuel in the game.

#### Scenario: Fuel from shale
- **WHEN** a powered crusher, a complete distillation column and a refinery are fed petrogen shale
- **THEN** petrogen fuel comes out of the refinery, and tar and gas come out of the column

### Requirement: Uranium processing
Uranic crust SHALL be crushed, leached, precipitated into yellowcake, and enriched in a centrifuge, which SHALL mostly yield depleted uranium and rarely enriched uranium, deterministically for the same factory.

#### Scenario: Enrichment
- **WHEN** a centrifuge processes 200 batches of yellowcake
- **THEN** it yields mostly depleted uranium and some enriched uranium, and the same world gives the same results

### Requirement: Ore washing
A washing plant SHALL wash fractal residue with water, each batch rolling a seeded loot table that is mostly grit and very rarely strange matter.

#### Scenario: Washing
- **WHEN** a washing plant washes 100 batches of residue
- **THEN** it yields mostly grit and some scrap, with the same results for the same world

### Requirement: Every byproduct has a use
Every byproduct of these chains SHALL be an ingredient or fuel somewhere.

#### Scenario: Byproducts
- **WHEN** the crafting check lists items with no use
- **THEN** no byproduct of these chains is listed
