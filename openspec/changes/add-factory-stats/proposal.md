# Proposal

## Why

The pole info box shows one moment: this step's supply and demand. That can't answer "is my generator keeping up?" or "where are my logs going?". Factorio answers both with graphs over time: a power graph per network and production statistics per item. A base with a tree farm, generators and belts needs the same view to tune.

## What Changes

- **Power history:** each network records its supply, demand and satisfaction once per game second, for the last 10 game minutes.
  - The pole info box gains an ASCII graph of the last 60 seconds, with a key to switch to 10 minutes.
  - It shows supply against demand, and marks seconds when the network was short.
- **Item flow:** the game counts every item produced and consumed, by item and by source.
  - Sources: mining or chopping by the player, machines (by kind), crafting, burning in generators, and planting.
  - The counts are kept per game minute for the last 60 minutes.
- **Production panel:** a new Production tab lists items with their produced and consumed rates (per minute, averaged over the last 1 or 10 minutes, or the last hour). Selecting an item shows its sources and a small graph.
- History and counts are saved with the world, so a reload changes nothing.

## Capabilities

### New Capabilities
- `factory-stats`: Power history per network, item flow counting by source, and the graphs and panel that show them.

### Modified Capabilities
<!-- None -->

## Impact

- **Core:** a `stats` module. `PowerNetworks` gains per-network history ring buffers, keyed by network id (the lowest pole position, stable across rebuilds). Item flow counters on `Resources` are updated where items are created or destroyed: `crafting::mine_tile`, machines, `power::draw`, crafting and planting.
- **Client:** the graph in the pole info box, a Production tab, and keybinds for the time range.
- **Saves:** new `Resources` fields (saved automatically).
- **Depends on** the power grid and tree automation.
