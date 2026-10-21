# LithicRivers tech tree

How the player gets from bare hands to nuclear power, and which OpenSpec change builds each step. It borrows mechanics from Minecraft mods (GregTech's long processing chains, Tekkit's nuclear tech, Big Dig's mining focus, Create's visible machinery), Factorio and Caves of Qud (see [INSPIRATION.md](INSPIRATION.md)). The blocks, materials and lore are LithicRivers' own: an older, more advanced world left wreckage behind, and the player builds on it.

## Rules every tier follows

- **Everything is craftable from the world.** Every placeable item has a recipe that bottoms out in raw sources (mining drops and creature loot). A test enforces this (`crafting::unobtainable`), so a new block can't ship without a way to make it.
- **Deterministic.** Ore, ruins and spouts are placed from the world seed and chunk coordinates. Factory progress depends only on elapsed game time.
- **Simulated everywhere.** Factory blocks keep working wherever they are, and chunks a machine reaches are generated exactly as if the player had walked there.

## Tiers

### 0. Scavenging (done)
Hands and the starting kit. Trees give logs and acorns. Rock gives stone, scrap tiles give iron scrap and scrap electronics, and feral dogs drop leather and meat. Recipes: planks, sticks, nails (iron scrap), string (leather), torches, and chests.
- `add-factory-foundation` (archived): factory blocks, chests, rotation, pass time.
- `add-crafting-tree` (archived): raw sources, the reachability test, the ingredient tree view.

### 1. Wood automation (in progress)
Turn the forest into a machine: farm trees, move items on belts, burn wood for power, smelt wood into denser fuel.
- `add-tree-farming` (archived): planting and growth.
- `add-conveyor-belts` (archived): belts and how blocks hand items to them.
- `add-power-grid`: burner generators, poles, networks.
- `add-crude-furnace`: unpowered smelting, biochar.
- `add-tree-automation`: auto-planters and auto-harvesters.
- `add-electric-furnace`: powered smelting.
- `add-view-modes`: views for belts, power and more.

### 2. Molten metal (planned)
Metal doesn't come from veins you dig out. Molten ore spouts well up liquid ore, mostly underground and sometimes on the surface, until their reservoir is full. The player scoops ore with a crucible early on, then pumps it through pipes into casting basins that turn it into ingots.
- `add-ore-spouts`: spouts (iron first), fluids, pumps, pipe networks, casting basins, crucibles, iron ingots.
- Later: copper and other spouts, alloy mixing from two fluids, processing machines that take fluid from pipes.

### 3. Ruins and thermoelectrics (planned)
Rare ruined reactors from the old world hold nuclear waste and a little uranium. A thermoelectric generator turns one radioactive item into small, endless power: the first power source that needs no fuel.
- `add-ruined-reactors`: ruin layouts, casing, waste, uranium deposits, the thermoelectric generator.

### 4. Advanced nuclear (idea, not yet proposed)
- Refining uranium into fuel through fluid processing (pipes from tier 2).
- Fuel rods and a reactor the player builds as a multiblock, with heat to manage and coolant moved through pipes.
- Meltdowns that damage terrain and the player's body.
- Radiation near waste and unshielded reactors, and protection from it.
- Reprocessing waste back into something useful.

## Adding a tier or a machine

1. Propose it with `/opsx:propose`, naming the tier it belongs to.
2. Give every new placeable item a recipe; the reachability test fails otherwise.
3. Add new raw materials to the mining drop table (or creature loot), never as special cases.
4. Update this file: move the step from "planned" to "done" when its change is archived.
5. Give every new factory block an example in the Factory showcase (`showcase::EXAMPLES` in `crates/core/src/showcase.rs`); `every_factory_block_kind_has_an_example` fails and names any kind without one.
