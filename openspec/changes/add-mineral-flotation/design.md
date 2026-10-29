# Design

## Context

- Machines (`machines.rs`, `furnace.rs`) already share a pattern: an inventory with input and output slots, progress counted in factory steps, power drawn from the grid, input from belts and extractors, and output pushed out the front. Furnaces share one smelting table.
- `add-ore-spouts` (not yet implemented) brings the fluid model (fluid kind, amount in mB, capacity), pumps, pipe networks and a "machine fluid input" that later machines plug into. This change builds on it and can't land before it.
- Rock drops 1 stone when mined, and there is no other kind of rock yet. `WoodenShavings` already drops from tree work and smelts into biochar.
- Randomness elsewhere (drops, worldgen) is seeded from the world seed and coordinates, so replays match (`AGENTS.md`).

## Goals / Non-Goals

**Goals:**
- A chain long enough to be a factory project (seven blocks, four fluids), where each step is visible and can be fed by belts and pipes.
- Copper and gems that can only be obtained by building it, and iron that's cheaper per minute here than from spouts once the line is built.
- Byproducts that all have a use, and one loop (tailings back to water) that rewards closing it.

**Non-Goals:**
- Different kinds of rock (granite, graphite and so on) with their own results. Stone is the only input for now; the flotation table is the place to vary later.
- Aquifers or water placed by worldgen. The well pumps water anywhere.
- Using copper wire, sapphire or alexandrite in other machines. Later changes do that.
- Apples, fruit pulp or other frothers. Tannin is the only frother; `add-crops`' mash could become a second one later.

## Decisions

**One seeded roll per batch, not fixed shares.** A flotation batch yields a single item drawn from the table, seeded from `(world seed, tank position, batch count)`, the same way drops are seeded. The alternative, fixed fractions with fractional carry-over, is predictable but dull, and it makes rare gem grit feel like bookkeeping. Rolls keep the gem moments, and 1000-batch tests pin the shares.

**Rates and yield.** One stone becomes 1 gravel, then 1 sand, then 2 dust. Four dust make one 250 mB batch, so a batch costs 2 stone. On average that's 0.45 copper, 0.40 iron and 0.10 gem grit per batch, so 10 stone becomes about 2.2 copper ingots and 2 iron ingots. Batch times are crusher 2, mill 3, mixer 2, tank 4 and sieve 2 factory seconds, and settling 20; a factory second is 3 game seconds. One tank therefore runs 5 batches a game minute, about 2 iron ingots. An iron spout fills about 15 ingots' worth a minute (25 mB per game second, `add-ore-spouts`), so one tank doesn't beat a spout. Flotation pays off differently:
- It is the only source of copper and gems.
- It turns stone, which every mining trip leaves behind, into metal anywhere, with no spout nearby.
- It scales by adding tanks.

That's how this design reads the inbox note's "more efficient": more out of boring stone, for the player who builds it out. The numbers are tuned in tests and the showcase, not in the spec.

**Two fluid inputs on the flotation tank.** Slurry enters from the back and tannin from either side, each as a separate machine fluid input. That's the only machine here with two fluids. A single mixed input would force the player to blend slurry and tannin in one pipe, but pipe networks hold one fluid kind each (`add-ore-spouts`). Tailings leave from the front pipe, and the solid result goes into the tank's output slot, where an extractor picks it up. Output to a belt would need a second front.

**The crusher is shared with `add-advanced-processing`.** It's one block with a recipe table, like furnaces. This change adds stone → gravel and gravel → sand. `add-advanced-processing` adds petrogen shale → petrogen slurry. Whichever change lands first creates the block, and the second only adds its rows.

**Tanned leather gates the flotation tank.** The tank's recipe needs tanned leather (for its seals), so the player builds a tannery and handles tannin before flotation is possible. It also gives dog and sheep leather a new use. Every other block is made from planks, stone, iron scrap, nails and iron ingots, so nothing in the chain needs its own output except tanned leather.

**Water well on any layer, powered.** It's simple and works everywhere. Rain collectors or aquifers can come later; the well isn't in their way.

## Risks / Trade-offs

- [Depends on unimplemented fluids] → The tasks start after `add-ore-spouts` is archived, and the first task checks its machine fluid input supports two inputs per block.
- [Too many new items crowd the Item Dictionary and the crafting list] → Intermediates (gravel, sand, dust, concentrates, grit) share one sprite family and sort together.
- [Balance] → The showcase line and a 1000-batch test give real numbers to tune. Rates are constants in one place.
- [Save size: seeded counters per machine] → One `u64` batch count per tank and sieve, saved with the machine component.

## Migration Plan

New blocks and items only. Old saves load unchanged, and every new component is registered in `saved_components!`.
