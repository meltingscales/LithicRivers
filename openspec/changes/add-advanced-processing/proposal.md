# Proposal

## Why

The Fractal Tear's materials (`add-fractal-tear-biome`) are worthless raw. The user wants the endgame to be about long, interesting processing chains, like the Create mod's diesel refining or GregTech's ore processing, not about mining more. Ore washing for very rare finds gives every leftover sludge a use.

## What Changes

- **Petrogen distillation** (inspired by Create's oil and diesel):
  - A **crusher** turns petrogen shale into petrogen slurry.
  - A **distillation column** (a 1-by-3 multiblock) heats the slurry and splits it into fractions by height: heavy tar at the bottom, petrogen oil in the middle, light petrogen gas at the top.
  - A **refinery** turns petrogen oil into **petrogen fuel**: the densest fuel in the game, for generators and power cells.
  - Tar becomes sealant, an ingredient for endgame machines.
- **Uranium processing** (inspired by GregTech and Factorio's nuclear mods), in steps:
  1. crush uranic crust
  2. leach it with acid
  3. precipitate the leachate into yellowcake
  4. enrich the yellowcake in a centrifuge, which mostly yields depleted uranium and rarely enriched uranium

  Enriched uranium feeds the nuclear tier (`add-ruined-reactors`' reactor and fuel rods).
- **Ore washing:**
  - A **washing plant** washes fractal residue (and any sludge byproduct) with water.
  - Each wash rolls a seeded loot table: mostly grit, sometimes scrap, very rarely a material found nowhere else (for example "strange matter", an endgame catalyst).
- **Byproducts:**
  - Every step has some (grit, depleted uranium, light gas).
  - Each byproduct has at least one use: gas burns in a gas generator, grit makes fill blocks, depleted uranium makes dense armor plates later.

## Capabilities

### New Capabilities
- `advanced-processing`: Petrogen distillation, uranium processing and ore washing, with their machines, items, byproducts and loot.

### Modified Capabilities
<!-- None: these are new machines; fuels join the existing fuel table. -->

## Impact

- **Core:**
  - New machines (crusher, distillation column, refinery, leaching tank, precipitator, centrifuge, washing plant), each powered, using `machines.rs`'s patterns.
  - Fluids for slurry, oil, gas, acid and water, from `add-ore-spouts`' fluid model.
  - New items, recipes, seeded loot tables for washing, and fuel values.
- **Client:** machine windows (inputs, outputs, fluid levels, progress), sprites, and showcase examples of each chain.
- **Depends on:**
  - `add-fractal-tear-biome` (the raw materials)
  - `add-ore-spouts` (fluids, pipes and pumps)
  - the power grid
  - `add-ruined-reactors`, which uses the enriched uranium
