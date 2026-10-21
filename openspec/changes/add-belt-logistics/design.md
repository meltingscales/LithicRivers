# Design

## Context

- `factory::accepts` / `insert` hand items to a block. Nothing takes items from one yet.
- Belts run last in each factory step, ordered so a full line shifts in one step.
- `TileKind::FactoryBlock` is not passable, whatever the block is.

## Goals / Non-Goals

**Goals:** move items between chests, machines and belts; sort and split; walk across belt lines.

**Non-Goals:** long-armed or stack extractors, underground belts, powered extractors (a later tier).

## Decisions

### Taking items
`factory::take(world, source, filter: Option<ItemKind>) -> Option<ItemKind>`: a chest gives its first matching stack, a harvester its output, a generator its fuel, and a planter its acorns. A belt gives nothing (extractors don't pull from belts).

### Extractor
- Moves one item every 1 game second (5 steps) from behind to in front, through `take` then `insert`. If `insert` refuses, the item stays where it was: a take is only committed after the insert succeeds.
- The filter extractor stores `filter: Option<ItemKind>`, set from a window listing the items in its source, or any item.

### Router
- Has one input (behind) and three outputs (left, straight, right).
- Each step, an item that arrived from its input goes to the next output in turn that accepts it.
- With a filter, the matching kind goes to the filtered side and everything else straight on.

### Step order
Growth, machine demand, power, machines, extractors, routers, belts.

### Walking on belts
- `FactoryBlock` tiles become passable for the player when the block is a belt. Passability needs the factory index, so movement checks `Game::is_walkable(pos)` instead of the tile alone.
- In the belt step, a player on a belt is moved one tile in its facing direction every `BELT_STEPS_PER_TILE` steps if the next tile is walkable and free. Creatures are not carried, to keep AI pathing simple.

## Risks / Trade-offs

- [A player carried into combat range] → Same as walking; combat starts as usual.
- [Passability now depends on blocks] → One function owns it; existing movement tests cover the rest.

## Implementation Notes

- **Module:** `crates/core/src/logistics.rs`, with an `Extractor { filter, progress }` component (shared by plain and filter extractors) and `Router { item, next, filter }`.
- **Taking** is `peek` then `remove_one`. An extractor commits the take only after `insert_from` into its front block succeeds, so a refused item never leaves its source.
  - Sources: a furnace's output; the first matching stack of any block's `Inventory` (chests, generator fuel, planter acorns, harvester output).
  - Belts give nothing.
- **Extractors** pass items through and never accept them themselves. A filter extractor with no filter does nothing.
- **Routers** hold one item and accept only from behind (the item moving along the router's facing), only while empty. Without a filter they try left, straight and right starting from the output after the last one used. **With a filter, the filtered kind always goes left and everything else straight on:** there is no per-router filter side, which keeps the window to a single pick.
- **Step order:** growth, machine demand, power, machines, furnaces, extractors, routers, belts, then carrying the player.
- **Walking on belts:** `logistics::walkable` (a passable tile, or a belt) replaces the tile check in player movement. Carrying uses `Resources::belt_ride_steps`, which counts factory steps on a belt and moves the player every `BELT_STEPS_PER_TILE` steps, if the next tile is walkable and holds no `BlocksMovement` entity. Creatures aren't carried.
- **Filter window:** the filter picker lists "(no filter)" and every item, marks the current one with `*`, and sets it on Activate.
- **Glyphs:**
  - extractor: `→ ← ↑ ↓` in amber
  - filter extractor: the same arrows in cyan
  - router: `╬`
  - Art follows `docs/ART-DIRECTION.md`.
- **Recipes:**
  - Extractor: 2 planks, 1 iron scrap, 2 nails.
  - Filter Extractor: 1 extractor, 1 scrap electronics.
  - Belt Router: 2 wooden conveyors, 1 iron scrap, 1 scrap electronics.
- **Showcase template updated:**
  - New "Extractor: chest to chest" and "Belt router split three ways" examples.
  - The tree farm became a "Self-feeding tree farm": the harvester fills a chest, and a filter extractor set to acorns feeds the planter. A 30-minute test checks that the planter runs out of its starting acorns and is refilled from its own harvest.
  - `look-at-sapling.txt` now teleports next to the farm's planted sapling instead of counting cursor steps from spawn.
