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

## Implementation Notes

- **`crates/core/src/plants.rs`** owns planting (`plant`), growth (`grow_saplings`, run as the first system in each factory step) and breaking (`break_sapling`). Saplings aren't factory blocks, so they aren't in the factory index; the few lookups by position query `Growth` directly.
- **Exact timing:** `run_due_steps` now passes each step its own tick (the start of the elapsed span plus 20 ticks per step), so a sapling grows in the first step at or after its `grows_at_tick`, however long the action was.
- **`Growth` remembers the tile it was planted on** (`planted_on`), so pulling up a sapling restores grass or dirt. `Growth` is registered for saving and is in the save-completeness worlds.
- **Placement:** `TileKind::from_item_kind(Acorn)` is `Sapling` so acorns count as placeable (hotbar picker, crafting reachability), but the client plants them through `plants::plant` rather than setting the tile. Acorns are a raw source, so reachability is unaffected.
- **Hotbar picker order:** the picker lists placeable items in inventory order, so with the starting kit the acorn comes before a chest. Scripts and docs that assign a chest now move down twice.
- Mining a sapling pulls it up ("You pull up the sapling. (+1 Acorn)").
