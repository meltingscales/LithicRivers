# World generation

How LithicRivers builds its world, layer by layer. The code is in `crates/core/src/world.rs`; the tests that pin it are in `crates/core/src/worldgen_tests.rs`.

## Chunks and layers

- **A chunk is one layer of 64 by 64 tiles,** keyed by `(cx, cy, cz)`. `CHUNK_SIZE = 64` and `CHUNK_SIZE_Z = 1`, so `cz` is the layer `z` itself.
- **Coordinates:** `cx = div_floor(x, 64)` and `tx = mod_floor(x, 64)`, and the same for y. Negative coordinates work.
- **Access:**
  - `GameWorld::get_tile_cached(x, y, z)` generates and keeps a chunk the first time it's read.
  - `set_tile_cached` writes a tile, generating the chunk first.
  - `get_tile` reads without keeping the chunk.
  - `get_tile_cached_no_worldgen` never generates; rendering uses it for chunks not loaded yet.
- **Saves** keep every generated chunk, so a chunk is generated once per world. Changing the generator changes only chunks explored afterwards; old worlds can show seams, which is accepted.

## Depth bands

`depth_band(z)` decides what a layer is. Worldgen, creature spawns and later features all ask it.

| Band | Layers | What generates |
|---|---|---|
| Sky | z ≥ 1 | Open air, to build up into with stairs and vertical conveyors |
| Surface | z = 0 | The biomes (plains, forest, rocky), tree clusters and rare one-tile resources, exactly as before depth bands existed |
| Shallows | z = -1 to `SHALLOWS_BOTTOM` (-5) | Rock with winding tunnels and chambers, and the first deposits |
| Deep | below -5 | The Lithic Rivers caverns (river channels, banks, rims, caverns), richer in deposits |

- `generate_chunk` dispatches on the band:
  - **Sky:** fills the chunk with air.
  - **Surface:** `generate_surface` runs the original code path unchanged. `the_surface_is_generated_as_before_depth_bands` pins a hash of its tiles.
  - **Shallows and deep:** `generate_underground` builds the terrain, then runs the feature stage.
- **Pure terrain:** `underground_tile(noises, x, y, z)` is the terrain of one underground tile before features. It is pure, so a neighboring layer can be checked without generating it.
- **Caves line up across layers:** shallow tunnels sample noise with a small z scale (`CAVE_Z_SCALE`), so 50 to 90% of a layer's open tiles are open on the next layer too. The deep caverns' noise varies slowly with z as well.

## The feature stage

After terrain, features run in a fixed order. Each one that rolls dice gets its own generator from `feature_rng(cx, cy, cz, Feature)`, seeded by the world seed, the chunk and the feature, so adding a feature never shifts another's rolls.

1. **Deposits** (noise, no dice): each `Deposit` in `DEPOSITS` has a vein-noise field and a threshold per band. Rock becomes the resource where the noise crosses the threshold, which makes clusters rather than single tiles.
   - Iron scrap and scrap electronics in the shallows and deep; plasteel scrap in the deep only.
   - Every deposit tile is solid and has a mining drop. `every_generated_solid_tile_can_be_mined` checks this for every tile kind the generator makes.
2. **Natural stairways:** for each pair of underground layers, `stairway_spots` picks up to two tiles per chunk that are open on both layers, and both layers place stairs there.
   - Both chunks compute the same spots, so the stairs always line up.
   - Stairways stop at z = -1, so the surface stays unchanged.
   - The way to the surface is the SapienCorp bunker's staircase, a column of stairs from z = -19 to z = 0 (`the_bunker_staircase_reaches_the_surface`).

## Structures

- **Placement:** structures (`.lrstructure` folders under `crates/client/assets/structures/`) are queued as pending structures and placed when their chunk is first ensured through `Game::ensure_chunk_with_pending_structures`.
- **Layers:** a buried structure's top layer is at its `z` and its bottom layer `layers - 1` below.
- **Tiles:** structure tiles marked `existing_worldgen` (`.`) keep the generated terrain.

## Creatures

`systems::creature_spawn_system` rolls each underground chunk once, when the player first comes within a chunk of it. It uses its own seeded generator, and the chance depends on the band (`slime_chance_percent`). The rolled chunks are saved.

## Tuning

- `worldgen_tests::print_the_survey` prints each band's tile mix for seed 12345, using `GameWorld::survey(z, half)`. Run it with `cargo test -p lithicrivers-core --lib print_the_survey -- --nocapture`.
- The other tests in `worldgen_tests.rs` pin the ranges: open space, deposit shares, overlap between layers.
- Change a constant, read the survey, and update the pinned ranges in the same change so reviewers see the effect.
- **Debug worlds:** the ore-test world is flat grass on top with the real bands below, for looking at caves and deposits. `scripts/snapshots/worldgen-depth-bands.txt` shows the shallows, the deep and the sky with fog of war off.
