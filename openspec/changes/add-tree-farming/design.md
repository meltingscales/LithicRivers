# Design

## Context

- Chopping a tree sets the tile to `Dirt` and drops 2–3 logs and 1–3 acorns (seeded by seed + position).
- `add-factory-foundation` provides factory steps by elapsed ticks (100 ticks = 1 game second), simulation wherever placed, and the position index.

## Goals / Non-Goals

**Goals:** renewable trees with deterministic growth.

**Non-Goals:** growth stages beyond sapling → tree, light or water requirements, other plant types, fertilizer.

## Decisions

### Sapling = tile + entity
`TileKind::Sapling` (passable, glyph `,` in green) marks the tile; an entity with `Position`, `Growth { grows_at_tick }` carries the timer and is indexed like a factory block. Storing the absolute `grows_at_tick` instead of a counter makes growth exact regardless of step size.

### Growth time
`grows_at_tick = planted_tick + (3..=5 minutes in ticks)` chosen by `GameWorld::new_rng(x, y, z)` (the same seeded RNG tree drops use).

### Growth system
Runs first in each factory step: for every `Growth` whose `grows_at_tick <= step_tick`, set the tile to `Tree` (through `Game::factory_tile`, so chunk edges are safe) and despawn the entity.

### Planting
`TileKind::from_item_kind(Acorn)` returns `Sapling`, with an extra rule in placement: the target tile must be `Grass` or `Dirt` and empty. Acorns stay usable in recipes.

## Risks / Trade-offs

- [Players plant whole fields and slow the game] → Growth is a cheap comparison per sapling per step.
- [Grown trees can block paths] → Same as natural trees; the player chooses where to plant.
