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
