# Design

## Context

- **Structures** are `.lrstructure` files under `crates/client/assets/structures` (plans or hand-written layers, see `docs/STRUCTURES.md`), queued as `PendingStructure`s and applied when their chunk is first ensured. A ruin queued in one chunk is placed whole across chunk edges.
- **Mining drops** come from `crafting::mining_drop` (from `add-crafting-tree`), and raw sources are derived from it.
- **Power** comes from `add-power-grid`: generators supply energy into networks of poles, and consumers draw a fraction.
- **Iron ingots** come from `add-ore-spouts`.

## Goals / Non-Goals

**Goals:** findable ruins that feel like LithicRivers' backstory; a fuel-free low-power source; materials for the advanced nuclear tier.

**Non-Goals:**
- Radiation damage or protection.
- Meltdowns.
- Player-built reactors.
- Refining uranium.

Each of these is listed in `docs/TECH-TREE.md` as a later change.

## Decisions

### Placement
- Like the spout chance, one constant sets the chance per surface chunk.
- Positions use `new_rng(chunk)` and are queued as `PendingStructure`s, so the existing pending-structure path generates them identically for players and machines.
- Ruins are surface structures, partly buried (`bury_structure`) for variety.
- The layout is picked from three to start, seeded the same way.

### Tiles and drops
| Tile | Breaks into | Drop | Mining |
|---|---|---|---|
| ReactorCasing | Air | 1 reactor casing scrap | 5× rock's cost |
| NuclearWaste | Air | 1 nuclear waste | normal |
| UraniumDeposit | Air | 1 uranium chunk | normal |

Mining cost per tile is new: `MiningDrop` gains a `cost_multiplier`, and the mining action's tick cost scales with it.

### Thermoelectric generator
- A `FactoryKind::Thermoelectric` with `Thermoelectric { source: Option<ItemKind> }`, registered as a generator in the power grid's supply step.
- Output is a constant per step (tunable): nuclear waste gives one third of a burner generator's maximum, and uranium gives two thirds.
- It never consumes its source, and it has no buffer: it supplies the same every step.
- The generator window reuses the two-panel transfer window, with a one-slot "heat source" panel that only accepts radioactive items.

### Recipe
Thermoelectric generator = 4 iron ingots + 2 scrap electronics + 2 reactor casing scrap (tunable). The reachability test covers it.

## Risks / Trade-offs

- [Infinite power trivializes burners] → Low output per generator, and each generator needs its own radioactive item; ruins are rare.
- [Ruin layouts are hand-made assets] → Three small layouts to start; the uranium test checks every layout.
- [Mining cost multiplier changes mining for every tile] → Defaults to 1, so only casing changes.
