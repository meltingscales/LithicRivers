# Spec Delta

## Purpose

Turns common stone into copper, iron and gems through a chain of powered machines that use fluids (water, slurry, tannin, tailings), with a frother that comes from tree farms.

## ADDED Requirements

### Requirement: Crushing and milling
A powered crusher SHALL turn 1 stone into 1 gravel and 1 gravel into 1 sand, each in a fixed time. A powered ball mill SHALL turn 1 sand into 2 mineral dust in a fixed time. Both SHALL take input from belts, extractors or by hand, and push output out their front like other machines. Without power they SHALL make no progress.

#### Scenario: Stone to dust
- **WHEN** a powered crusher is fed 4 stone, its gravel is fed back to it, and the sand goes into a powered ball mill
- **THEN** the line makes 8 mineral dust

#### Scenario: Unpowered crusher
- **WHEN** a crusher holding stone has no power for 10 game minutes
- **THEN** it still holds the stone and has made nothing

### Requirement: Water well
A powered water well SHALL pump water into the pipe or fluid input in front of it at a fixed rate, on any layer, as long as there is room. Without power it SHALL pump nothing.

#### Scenario: Filling a pipe
- **WHEN** a powered water well faces an empty pipe network
- **THEN** the network fills with water at the well's rate until it is full

### Requirement: Slurry mixer
A powered slurry mixer SHALL combine 4 mineral dust with 250 mB of water into 250 mB of slurry, and output it to the pipe in front of it. It SHALL make nothing while it lacks dust, water, room for slurry, or power.

#### Scenario: Making slurry
- **WHEN** a powered mixer has 8 mineral dust and 500 mB of water, with an empty pipe in front
- **THEN** it makes 500 mB of slurry and has no dust or water left

#### Scenario: No water
- **WHEN** a powered mixer has dust but no water
- **THEN** it makes nothing and its window says it needs water

### Requirement: Tannery and tanned leather
A powered tannery SHALL steep 4 wood shavings in 250 mB of water to make 250 mB of tannin. With 100 mB of tannin it SHALL also tan 1 leather into 1 tanned leather.

#### Scenario: Shavings to tannin
- **WHEN** a powered tannery has 8 wood shavings and 500 mB of water
- **THEN** it makes 500 mB of tannin

#### Scenario: Tanning leather
- **WHEN** a powered tannery holding 100 mB of tannin is given 1 leather
- **THEN** it makes 1 tanned leather and uses the tannin

### Requirement: Flotation
A powered flotation tank SHALL take 250 mB of slurry and 50 mB of tannin per batch, and after the batch time produce one roll from a seeded table: copper concentrate (45%), iron concentrate (40%), gem grit (10%) or nothing (5%). It SHALL also produce 200 mB of tailings. The roll SHALL be deterministic from the world seed, the tank's position and its batch count. Without tannin, slurry, room for tailings or power, it SHALL not start a batch.

#### Scenario: A batch
- **WHEN** a powered tank with 250 mB of slurry and 50 mB of tannin finishes a batch
- **THEN** it holds one rolled result (or nothing) and 200 mB of tailings, and the slurry and tannin are used

#### Scenario: No frother
- **WHEN** a powered tank has slurry but no tannin
- **THEN** it starts no batch and its window says it needs tannin

#### Scenario: Same results every replay
- **WHEN** the same world runs the same flotation line twice
- **THEN** the tank produces the same results in the same order both times

#### Scenario: Yield over many batches
- **WHEN** a tank runs 1000 batches
- **THEN** the counts of copper, iron and gem grit are each within a few percent of their table shares

### Requirement: Sieving gems
A powered sieve SHALL sort 1 gem grit per cycle into a seeded result: sapphire (20%), alexandrite (5%) or nothing (75%). The roll SHALL be deterministic from the world seed, the sieve's position and its cycle count.

#### Scenario: Sieving
- **WHEN** a powered sieve sorts 100 gem grit
- **THEN** it yields some sapphires and fewer alexandrites, and the same counts on every replay

### Requirement: Settling tailings
A settling pond SHALL turn 1000 mB of tailings into 1 clay and 800 mB of water over a fixed time. It needs no power, and SHALL return the water to the pipe in front of it.

#### Scenario: Recycling water
- **WHEN** a settling pond receives 1000 mB of tailings and its settling time passes
- **THEN** it holds 1 clay and has sent 800 mB of water into the pipe in front of it

### Requirement: Products and recipes
The chain's products SHALL have these uses: 1 copper ingot crafts 2 copper wire; sapphire and alexandrite are kept as ingredients for later machines. Every new block SHALL have a recipe that bottoms out in raw sources, without needing anything the chain itself produces except tanned leather, which comes from a tannery. The flotation tank's recipe SHALL need tanned leather.

#### Scenario: Everything is craftable
- **WHEN** the crafting reachability test runs
- **THEN** every new block and item is reachable from raw sources

#### Scenario: Copper wire
- **WHEN** the player crafts with 1 copper ingot
- **THEN** they get 2 copper wire

### Requirement: Flotation in the showcase
The Factory showcase SHALL include a complete flotation line, from a chest of stone to copper ingots, that runs without the player's help.

#### Scenario: Showcase line
- **WHEN** the factory showcase world runs for 10 game minutes
- **THEN** the flotation example's output chest holds copper or iron ingots
