# Design

## Context

- `GameWorld::legacy_tile` builds the deep band (`BiomeBand::LithicRivers`) from a horizontal stripe field: a river centerline every 22 tiles with up to 5 tiles of meander. Tiles 0-2 from the centerline are the core (all air today), 2-3 the banks, 3-4 the rim (rock with a little iron and plasteel), and caverns beyond.
- The starting bunker sits at z = -19, inside the deep band. Structures overwrite generated tiles, and the way out is the bunker's own staircase column, so lava outside the bunker can't trap a new player.
- Natural stairways are placed only on tiles that are passable on both layers (`stairway_spots`), using the pure `underground_tile`.
- `PaletteKey::Lava` and a `fluids/lava` sprite already exist but nothing uses them.
- Plain blocks are placed by the client (`world_build.rs`) with `factory::can_place_at`, which requires a passable tile.

## Goals / Non-Goals

**Goals:** lava in the rivers, natural fords, player-made bridges, a rare deep-only material with a real use.

**Non-Goals:**
- Flowing or spreading lava, and lava damaging things. Lava is terrain.
- Lava giving off light. That needs a cheap way to find lit lava tiles near the player and can come later.
- Pumping lava (`add-ore-spouts` owns fluids).
- Mining bots (`add-mining-bots`).

## Decisions

### Lava is an impassable tile that sight passes over
`TileKind::Lava` is not passable, so movement, pathfinding, creatures, natural stairways and block placement treat it like a wall with no special cases. `blocks_sight` gets one exception so the player sees across a river.
Alternative: passable lava that burns body parts. Rejected for now: it would need an environmental damage path that doesn't exist, and pathing creatures and belts would need to avoid it. Impassable lava is simple and safe, and mining bots bring the danger.

### Lava fills the inner core; fords are seeded per stretch
The lava reaches 1.5 tiles from the centerline (`LAVA_HALF_WIDTH`); the rest of the 2-tile channel is open shore, so rivers stay walkable alongside. Each river has one ford of `LavaCrust`, 3 to 5 tiles wide, in every stretch of `FORD_SPACING` (48) tiles along x, at a spot hashed from the world seed, the stretch, the river and the layer (`GameWorld::river_ford`). A crossing is never more than 96 tiles away.
Alternative: fords where a slow Perlin noise dips. Rejected after tuning: the worst gap along a river stayed above 140 tiles at any setting that kept fords rare.

### Rivers south of y = 0
The stripe field used `fract()`, which is negative for negative y, so the deep had no rivers south of y = 0. It now uses `rem_euclid(1.0)`. This only changes deep chunks explored after the change.

### Bridging goes through one core helper
`factory::place_plain_block(world, res, tile, pos)` places a plain block tile. On lava it sets `LavaCrust` instead and returns that, so the client logs "The block sinks, and the lava crusts over." The client's plain-block path calls it instead of doing the check and the write itself. `can_place_at` stays unchanged (factory blocks and blueprints still refuse lava).

### Riverglass forms on the rims
On rim tiles (distance 3-4) that are rock after the existing iron and plasteel enrichment, a separate riverglass noise above a high threshold makes `TileKind::Riverglass`. It is solid and drops 1 riverglass. The share is pinned in `worldgen_tests` (roughly 0.1-0.6% of deep tiles), and it never appears in the shallows or on the surface.

### Recipes
- Teleporter: add 4 riverglass (its comment already says the ingredients are placeholders).
- Advanced power cell: add 2 riverglass.
`raw_sources_are_what_the_world_drops` gets riverglass. The crafting reachability tests must still pass.

### Old saves
Generated chunks are saved, so explored chunks keep empty rivers. No migration.
