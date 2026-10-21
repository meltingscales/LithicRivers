# Proposal

## Why

Metal in LithicRivers today comes only from scattered scrap tiles. A metal tier needs a steady, interesting source, and one that's unique to this game rather than another ore vein to dig out. Molten ore spouts are that source. They are permanent features, mostly underground and sometimes on the surface, that well up liquid ore, and the player has to pump the ore out and cast it before it's useful. That gives mining a factory problem to solve instead of a pickaxe loop. It's the first step of the LithicRivers tech tree laid out in `docs/TECH-TREE.md`.

## What Changes

- **Molten ore spouts:**
  - Worldgen places these unbreakable blocks underground, and on the surface at half the underground chance, deterministically from the world seed and chunk.
  - Each spout produces one molten ore (iron to start) into its own reservoir at a fixed rate, up to a cap, then stops until it's drained.
  - Mining one does nothing and says it can't be broken.
- **Fluids:**
  - A small fluid model shared by every block that holds liquid: a fluid kind, an amount in millibuckets (mB) and a capacity.
- **Pump:** a powered factory block. Placed next to a spout (or any fluid block), it moves fluid from the block behind it into the block in front, at a fixed rate per factory step, while powered. The block in front can be a machine directly, or a pipe.
- **Pipes:**
  - Pipes carry fluid from pumps to machines for processing.
  - A connected run of pipes forms one network that holds one fluid kind, with 100 mB of room per pipe.
  - Each factory step the network feeds the adjacent machines that accept that fluid, in a fixed order, up to a throughput limit.
  - Pipes work across chunk edges, and breaking a pipe drops what it held.
- **Casting basin:** a factory block that fills with molten ore. Each full casting (100 mB) cools over a set time into one ingot, which is pushed out the front onto a belt, or into a chest. With nothing in front, ingots stay in the basin.
- **Hand scooping:**
  - Before power, a crucible lets the player scoop 100 mB from a spout by interacting with it.
  - Interacting with a basin while holding a full crucible pours it in.
  - This makes iron reachable early, slowly.
- **Machines take fluid from pipes:**
  - The casting basin is the first such machine.
  - Later processing machines take fluid through the same machine fluid input, so they plug into existing pipe networks.
- **New items:** iron ingot, crucible (empty and full), pump, pipe and casting basin.
  - Recipes for the crucible, pump, pipe and basin follow the crafting-tree rules: every placeable item must be craftable.
  - Iron ingots become an ingredient for later machines.
- **Rendering:**
  - A spout glows and shows its fill level in Look mode.
  - Basins show casting progress.
  - Pumps show their facing.

## Capabilities

### New Capabilities
- `ore-spouts`: Where spouts appear, how they fill and stop, that they can't be broken, scooping, and casting into ingots.
- `fluids`: Fluid amounts and capacities, the pump, pipe networks, and how machines take fluid from pipes.

### Modified Capabilities
<!-- None yet. Adds recipes to the crafting table owned by `crafting` (when `add-crafting-tree` is archived first, this change adds a MODIFIED delta listing the new raw-free recipes). -->

## Impact

- **Core:**
  - New components: `FluidTank { fluid, amount_mb, capacity_mb }` on factory blocks, `OreSpout { ore, rate_mb_per_step }`, `Pump`, `Pipe` and `FluidInput`. `CastingBasin` reuses the furnace-style progress fields.
  - Factory systems run in this order each step: spouts fill, pumps move fluid, pipe networks deliver, then basins cast.
  - Worldgen gets a seeded spout placement pass for underground chunks.
  - Mining refuses spouts.
  - New `ItemKind`s, `FactoryKind`s and recipes, all registered for saving.
- **Client:** sprites for the spout, pump, pipe (drawn joined to neighbors), basin and items; scooping and pouring through Interact; fill levels in Look mode.
- **Dependencies:**
  - `add-factory-foundation` (done) and `add-crafting-tree`.
  - `add-power-grid` for the pump.
  - `add-conveyor-belts` is optional: basins push onto belts when they exist, otherwise into chests.
