# Proposal

## Why

LithicRivers is set in the wreckage of an older, more advanced world. Ruined nuclear reactors make that history something the player can find and use. They are rare, dangerous-looking multiblock ruins that hold nuclear waste and the occasional uranium chunk. Those materials power the first generator that needs no fuel: a thermoelectric generator, which gives low power forever. It's the step after ore spouts in the LithicRivers tech tree (`docs/TECH-TREE.md`), and it opens the advanced nuclear tier later.

## What Changes

- **Ruined reactors:**
  - Rare multiblock ruins placed by worldgen, deterministically from the world seed and chunk.
  - Several layouts, from a cracked containment ring to a half-buried core.
  - Built from reactor casing, which is very hard to break and drops scrap; nuclear waste blocks, which drop nuclear waste; and a few uranium deposits, which drop uranium chunks.
  - A ruin is generated whole even when it spans several chunks.
- **New items:** nuclear waste, uranium chunk, thermoelectric generator, reactor casing scrap.
- **Thermoelectric generator:**
  - A factory block that holds one radioactive item as a heat source.
  - While loaded, it produces a small, constant amount of power forever, and never uses up the item: about a third of a burner generator with nuclear waste, about twice that with a uranium chunk.
  - It joins the power grid like any generator.
- **Recipes:** the thermoelectric generator is crafted from iron ingots (from ore spouts), scrap electronics and reactor casing scrap, so this tier sits after ore spouts in the tree.
- **Advanced nuclear tier:** outlined only, in `docs/TECH-TREE.md` (refined fuel, reactors you build, meltdowns). Each piece will be its own change.

## Capabilities

### New Capabilities
- `ruined-reactors`: Where ruined reactors appear, what they're made of and what each part drops.
- `thermoelectric-power`: The thermoelectric generator, its heat sources and the power it gives.

### Modified Capabilities
<!-- None yet. Adds a generator type to `power-grid` (from add-power-grid); if that change is archived first, this change adds a MODIFIED delta for the generator list. -->

## Impact

- **Core:**
  - New tiles: `ReactorCasing`, `NuclearWaste`, `UraniumDeposit`, with mining drops in the crafting drop table so they're raw sources.
  - Ruin structures as `.lrstructure` files, placed through the existing pending-structure path, so the chunks a machine generates match the chunks a player walks into.
  - A `Thermoelectric { source }` generator component, plus new items, recipes and save registration.
- **Client:** tile and block sprites; a generator window showing the heat source and output.
- **Assets:** ruin layouts written as structure plans (`docs/STRUCTURES.md`).
- **Dependencies:** `add-power-grid` (generators and networks); `add-ore-spouts` (iron ingots for the recipe); `add-crafting-tree` (drop table, reachability test).
