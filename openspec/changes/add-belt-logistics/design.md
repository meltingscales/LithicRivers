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
