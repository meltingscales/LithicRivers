# Design

## Context

- `power::step_power` runs every factory step (5 per game second) and fills `Network { supply_w, demand_w }`.
- Network ids are the lowest pole position, so they survive rebuilds and loads.
- Items are created by `crafting::mine_tile` (player and harvester), by crafting, and by machines; they are destroyed by `power::draw` (fuel), crafting ingredients and `plants::plant`.

## Goals / Non-Goals

**Goals:** see power and item flow over time; exact and deterministic; cheap to record.

**Non-Goals:** per-machine graphs, exporting stats, alerts.

## Decisions

### Power history
- Every 5 factory steps (one game second), each network appends `(supply_w, demand_w)` to a ring buffer of 600 samples.
- Buffers are keyed by network id in a map. When networks merge, the merged network keeps the history of the lowest id; a network that disappears keeps its history until its id is unused for 10 minutes.

### Item flow
- `ItemFlow { produced, consumed }` counters, keyed by `(ItemKind, Source)`, per game minute, in a ring of 60 minutes.
- `Source` is an enum: `PlayerMining`, `Machine(FactoryKind)`, `Crafting`, `Fuel`, `Planting`.
- One function, `stats::record(res, item, qty, source, produced: bool)`, is called at each place that creates or destroys items. A test lists every such place by checking that a scripted session's inventory deltas match the recorded totals.

### Graph rendering
- ASCII bar columns with Unicode block characters (`▁▂▃▄▅▆▇█`), one column per sample (or per bucket for the 10-minute view), scaled to the larger of supply and demand. Short seconds are drawn in red.
- The renderer is a pure function `graph(samples, width, height) -> Vec<String>`, unit tested.

### Production tab
- A new tab after Crafting, with keybinds for the time range in `default_config.rs`.

## Risks / Trade-offs

- [Memory: 600 samples per network] → Small (two u64 each); stale networks expire.
- [A missed record call makes numbers lie] → The inventory-delta test catches it.

## Implementation notes

- **Module:** `factory_stats.rs`; the data is `Resources::factory_stats` (`res.stats` was already the achievement counters).
- **Power sampling:** once per factory second (every 5 factory steps, 3 game seconds), not once per game second: factory steps are 0.6 game seconds, so a game second isn't a whole number of steps. 10 game minutes is 200 samples and the short view's 60 game seconds is 20. Histories live in a list sorted by id rather than a map, since the save is JSON and positions can't be map keys.
- **Graph:** `ui::graph::bars` is a pure function. The pole box draws demand per sample, scaled to the larger of supply and demand, with short samples in red. `stats.CYCLE_RANGE` (`r`) switches between 60 seconds (one column a sample) and 10 minutes (bucketed to fit). The pole box got taller to fit it.
- **Item flow:** minute buckets hold a list of `(item, source, produced, consumed)` entries. `record` is called by:
  - player mining and chopping, and pulling up a plant (`PlayerMining`)
  - the harvester, furnaces and the fermenter, the spawner, the feeder, the processor and the bio-vat (`Machine(kind)`)
  - player crafting (`Crafting`)
  - generators and crude furnaces burning fuel (`Fuel`)
  - `plants::plant` (`Planting`)

  `recorded_totals_match_what_inventories_gained_and_lost` checks player mining, planting, fuel and a furnace against the inventories. Feeding a sheep by hand and items picked up from corpses aren't counted, as neither is production.
- **Production tab:** after Items (not after Crafting, where Items now sits), so earlier tabs keep their positions. It averages over the last 1, 10 or 60 game minutes, dividing by the whole window. The menu bar in every snapshot changed.

