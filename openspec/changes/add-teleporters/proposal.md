# Proposal

## Why

The world is large and quests are far apart: the SapienCorp factory is a mile from the bunker. Once a player has bases in several places, walking between them is dead time. Buildable teleporters, like Minecraft's Waystones mod, let a player link their own bases, with a real energy cost so they belong to the mid and late game and tie into the power grid.

## What Changes

- **Teleporter:** a mid-to-late-game factory block with a built-in battery bank that charges from the power grid it is connected to.
- **Naming:** a new teleporter gets a default name (such as "Teleporter 3") that the player can rename from its window.
- **Travelling:** interacting with a teleporter opens a list of every other teleporter in the world, by name, with its distance. Choosing one moves the player next to the destination.
  - Each jump costs a large amount of energy from the source teleporter's battery bank, rising with distance.
  - If the bank is short, nothing happens, and a message says how much more charge is needed.
- **Window:** shows the battery charge, the charge rate from the grid, and the cost to each destination.
- **Recipe:** later-tier materials (placeholder ingredients until those tiers exist).

## Capabilities

### New Capabilities
- `teleporters`: Teleporter blocks, battery banks charged from the grid, naming, the destination list, jump costs and the out-of-charge message.

### Modified Capabilities
<!-- None. The battery bank is an ordinary power consumer whose demand depends on its charge; the power-grid requirements hold unchanged. -->

## Impact

- **Core:** `FactoryKind::Teleporter` with `Teleporter { name, charge_j }`, charged as a power consumer up to its capacity, a jump function that checks the cost, moves the player and spends charge, and teleporters across chunks and z-levels.
- **Client:** the teleporter window (charge, destinations with distance and cost, rename) and a short arrival effect.
- **Showcase:** a pair of linked teleporters example.
