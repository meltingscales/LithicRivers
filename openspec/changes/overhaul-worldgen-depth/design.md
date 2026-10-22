# Design

## Context

- **Generator:** `GameWorld::generate_chunk(cx, cy, cz)` fills a 64 by 64 slice per layer (`CHUNK_SIZE_Z = 1`).
  - It uses Perlin fields for height, biome and features. `biome_for` returns `LithicRivers` for every z ≤ -5, and the surface biomes otherwise, at every height.
  - Rare resources are one tile per chunk, by chance.
  - Structures (the bunker at z = -19, the factory) are placed in `ensure_chunk`.
- **Mining drops:** `crafting::mining_drop`. Tiles not listed can't be mined.
- **Saves:** generated chunks are saved, so a changed generator only affects chunks generated afterwards.

## Goals / Non-Goals

**Goals:** a world that reads as surface, shallows and deep; deposits that are findable and mineable; a walkable underground; every feature deterministic from the seed and position; z = 0 unchanged.

**Non-Goals:**
- New biomes on the surface.
- Fluids, lava flow and ore spouts themselves (`add-ore-spouts`).
- Reactor ruins (`add-ruined-reactors`).
- Changing the bunker or factory structures.

## Decisions

### Bands
- `depth_band(z) -> Band { Sky, Surface, Shallows, Deep }`. The cut-offs are constants (`SHALLOWS_BOTTOM = -8`, tunable).
- z = 0 keeps today's code path exactly. The other bands get their own terrain functions.

### Caves across layers
- Cave noise samples `(x * s, y * s, z * s_z)` with `s_z` large enough that neighboring layers differ a little but overlap mostly (target: 60 to 80% of open tiles open on the layer above).
- The survey pins this overlap.

### Natural stairways
- In the feature stage of a chunk at layer z, with its neighbor at z + 1, tiles open on both layers are candidates. A seeded few become stairway pairs (target: about 2 per chunk where there are candidates).
- **The bunker route:** from the bunker's own staircase, a seeded path up to z = 0 is guaranteed. Each layer carves a short tunnel to the next stairway, so the quest can always be finished without crafting stairs.

### Deposits
- Each resource has `Deposit { tile, noise_seed, scale, threshold_by_band }`. A tile becomes the deposit where vein noise crosses the band's threshold inside rock, which makes clusters rather than single tiles.
- **Plasteel (decided by the user):** add `ItemKind::PlasteelScrap` and a mining drop for the tile. It fits the "salvaged advanced tech" lore, and later recipes can use it.
- Every deposit tile becomes solid (not passable) and gets a mining drop. A test checks that every generated tile kind is either passable terrain or mineable.

### Feature stage
- **Order:** deposits, natural stairs, creature spawns, then later spouts and reactors. Each feature gets its own generator, seeded from `(seed, chunk, feature id)`, so adding a feature never changes another's rolls.

### No generator versions (decided by the user)
- The new generator replaces today's. There is no `worldgen_version` and no frozen old generator.
- Old saves keep their saved chunks. Chunks explored after updating use the new generator, so seams can appear in old worlds. That's accepted while the game has few players.

### Survey
- `worldgen::survey(seed, layers, area)` returns the tile mix per layer.
- A test pins bands within ranges (for example, shallows open 15 to 35%, iron 0.3 to 1%), and a test prints the full table on failure.

## Risks / Trade-offs

- [Every normal-world snapshot changes] → z = 0 is bit-identical, so only scripts that go below or above the surface change, and those are reviewed one by one.
- [Seams in old worlds] → Accepted by the user; saved chunks still load, nothing is lost.
- [Natural stairways make vertical conveyors less special] → They are rare and only where caves meet; building your own is still the way to go where you choose.
- [Tuning by feel] → The survey makes each number visible; the user check plays the bunker escape on a few seeds.
