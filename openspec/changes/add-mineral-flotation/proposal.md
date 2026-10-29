# Proposal

## Why

Rock is the most common thing in the world and nearly useless: it drops stone, and stone builds little. The user wants a long processing chain in the spirit of GregTech's ore processing and Create's machinery, where huge amounts of plain stone become copper, iron and gems. It should be LithicRivers' own, use fluids, and tie into farming. Today copper and gems have no source at all, and the tech tree's next metals need one.

## What Changes

- **Mineral flotation: the only source of copper and gems.** It's also a second source of iron, which turns waste stone into metal anywhere and scales by adding tanks. (Decided by the user: sifting only, not copper spouts.)
- **The chain**, every step a powered factory block that belts and pipes can feed:
  1. A **crusher** grinds stone into gravel, and gravel into sand.
  2. A **ball mill** grinds sand into mineral dust.
  3. A **slurry mixer** stirs mineral dust into water, making slurry (a fluid).
  4. A **flotation tank** bubbles slurry with a frother. Each batch rolls a seeded result: copper concentrate, iron concentrate, or gem grit (rare). The rest leaves as tailings (a fluid).
  5. A **sieve** sorts gem grit into sapphire, alexandrite (rarer), or nothing but grit.
  6. A **settling pond** lets tailings settle into clay, and returns most of the water to its pipe.
  7. Any furnace smelts **copper concentrate into copper ingots** and **iron concentrate into iron ingots**.
- **Water:** a **water well** pumps water into its pipe at a fixed rate while powered, on any layer. It's the first water source in the game.
- **Farming tie-in: the frother is tannin.** A **tannery** steeps wood shavings, a by-product of tree farms, in water to make tannin (a fluid). Without tannin a flotation tank does nothing. Tannin also tans leather into **tanned leather**, which the flotation tank's recipe needs, so dogs and sheep feed the chain too. (Decided by the user: the frother comes from farms.)
- **First uses:** copper ingots make **copper wire**, the ingredient later tiers build on (power cells, SapienNet, the nuclear tier). Sapphire and alexandrite become ingredients for later optics and endgame machines. Clay bakes into **fired bricks**, a building block.
- **Showcase:** a complete flotation line from a stone chest to copper ingots, as a Factory showcase example.

## Capabilities

### New Capabilities
- `mineral-processing`: crushing, milling, slurry mixing, flotation with its seeded results, sieving, settling, the water well, the tannery, and the items they make.

### Modified Capabilities
- `smelting`: the shared smelting table gains copper and iron concentrate, plus clay into fired bricks.

## Impact

- Depends on `add-ore-spouts` for the fluid model, pumps, pipe networks, machine fluid inputs and the iron ingot. It must land first.
- Shares the **crusher** with `add-advanced-processing`. Whichever change lands first adds the block, and the other adds its recipes (stone and gravel here, petrogen shale there).
- Core: new `ItemKind`s (gravel, sand, mineral dust, copper and iron concentrate, gem grit, sapphire, alexandrite, clay, fired brick, copper ingot, copper wire, tanned leather and the seven blocks), new fluid kinds (water, slurry, tannin, tailings), machine logic in the patterns of `machines.rs` and `furnace.rs`, recipes, and Factory showcase examples.
- Client: machine windows, sprites for every new item and block (`docs/ART-DIRECTION.md`), Item Dictionary entries.
- Saves: new components registered in `saved_components!`.
- Docs: `docs/TECH-TREE.md` gains a "Mineral processing" tier after Molten metal and drops "copper spouts".
