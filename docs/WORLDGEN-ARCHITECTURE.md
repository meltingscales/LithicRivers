# World Generation Architecture (LithicRivers)

This document summarizes how world generation, chunking, saving/loading, and client prefetching work after the 3D refactor.

## Overview

* __3D addressing, 2D chunks__: The world space is 3D. Chunks are addressed by 3D keys `(cx, cy, cz)` but each chunk stores a 2D tile slice (X,Y) only. Different Z levels map to different chunk Z keys.
* __Noise-driven terrain__: Generation uses Perlin noise sampled in 3D `(x, y, z)` space. The biome and terrain selection depend on the Z slice being generated.
* __Explicit Z everywhere__: All tile reads/writes use `(x, y, z)`. Prefetching and systems pass an explicit `z` so the correct Z slice is generated and cached.

## Core Types and Constants

File: `crates/core/src/resources/world.rs`

* __`CHUNK_SIZE: i32 = 64`__: Chunk width/height in tiles. Z is sliced by chunk Z index; each chunk contains `CHUNK_SIZE * CHUNK_SIZE` tiles.
* __`World`__:
  - Fields: `seed: u64`, `gen_z: i32` (current generation slice), `chunks: HashMap<(i64, i64, i64), Chunk>`.
  - Purpose: generation, cache management, tile access and mutation.
* __`Chunk`__:
  - Internals: `tiles: Vec<TileKind>` sized `CHUNK_SIZE * CHUNK_SIZE` (2D only).
  - Accessors: `get(tx, ty)`, `set(tx, ty, t)`.

## Coordinate Conversions

* __World to chunk/local__:
  - `cx = div_floor(x, CHUNK_SIZE)`, `cy = div_floor(y, CHUNK_SIZE)`, `cz = div_floor(z, CHUNK_SIZE)`
  - `tx = mod_floor(x, CHUNK_SIZE)`, `ty = mod_floor(y, CHUNK_SIZE)`
* Helpers: `div_floor()` handles negatives correctly; `mod_floor()` returns a non-negative remainder in `[0, CHUNK_SIZE)`.

## Generation Flow

* __`ensure_chunk(cx, cy, cz)`__ generates and caches a chunk if absent.
* __`generate_chunk(cx, cy, cz, &mut chunk)`__ produces the 2D tile slice for the chunk at `(cx, cy, cz)`:
  - Samples 3D noise using world-space coordinates derived from `(cx, cy)` and local `(tx, ty)` plus the Z slice (`cz` or `gen_z`).
  - Selects biome via `biome_for(wx: f64, wy: f64, zf: f64)` with Z explicitly included.
  - Post-processes (e.g., trees) using seeded RNG derived from `seed`, `cx`, `cy`, and current `gen_z`.
  - Structure application currently applies only the first Z layer of a structure into this 2D slice via `apply_structure(chunk, structure, ox, oy)`.

## Tile Accessors

* __Read (no cache insert)__: `World::get_tile(x, y, z) -> TileKind`
  - Generates a local chunk transiently if not cached and returns the tile at `(tx, ty)`.
* __Read (cached)__: `World::get_tile_cached(&mut self, x, y, z) -> TileKind`
  - Ensures the chunk in cache then returns `chunk.get(tx, ty)`.
* __Write (cached)__: `World::set_tile_cached(&mut self, x, y, z, t)`
  - Ensures the chunk in cache then mutates `chunk.set(tx, ty, t)`.

## Prefetching and Z Slices

* __`World::prefetch_rect(left, top, right, bottom, z)`__
  - Computes `(min_cx..=max_cx, min_cy..=max_cy)` covering the 2D rectangle.
  - Uses `cz = div_floor(z, CHUNK_SIZE)` and ensures all relevant `(cx, cy, cz)` chunks.
* __`World::set_generation_z(z)`__
  - Sets `gen_z` and clears cache so subsequent generation reflects the new Z slice.

Client integration (file: `crates/client/src/main.rs`):
* Rendering sets `gen_z` from `view_z` each frame and prefetches via `prefetch_rect(..., view_z)`.
* Look mode also prefetches with the look cursor’s `z` to keep navigation smooth.

## Save/Load (3D Keys)

File: `crates/core/src/save_load.rs`

* __JSON mirror__ `WorldJson` stores `chunks: Vec<((i64, i64, i64), TileChunk)>` to encode the 3D HashMap keys.
* Round-trip:
  - Save: `WorldJson::from(world)` uses `world.chunks_to_vec()`.
  - Load: `TileWorld::from(WorldJson)` sets `gen_z` then `set_chunks_from_vec(chunks)`.
* __Fluids__ are encoded as `Vec<(Position, Fluid)>`; upon loading, they are restored into `FluidManager`.

## Fluids and World Interaction

File: `crates/core/src/resources/fluids.rs`

* __`FluidManager`__ stores fluids keyed by 3D `Position { x, y, z }`.
* `can_hold_fluid()` queries tiles via `world.get_tile(x, y, z)` so solidity tests are slice-accurate.
* `process_fluids()` and `process_fluids_clamped()` operate in 3D, including vertical flow to neighboring Z slices.
* Deterministic lava seeding for the Lithic Rivers biome occurs for chunks at/under certain Z (`gen_z <= -5`), tracked by `seeded_lava_chunks: HashSet<(i64, i64, i32)>`.

## Structures

* Structures are currently applied as 2D on a single Z layer per generated chunk slice through `apply_structure(chunk, structure, ox, oy)`.
* Future direction (see prior work on position-based seeding): move toward position-based seeding on a fixed 3D grid independent of `CHUNK_SIZE`, enabling consistent placement across chunk sizes and full multi-layer structures.

## Design Decisions

* __3D keys, 2D storage__: Keeps memory and tile operations simple while enabling vertical gameplay via stacking slices.
* __Explicit Z in APIs__: Prevents ambiguity and ensures consistent behavior for rendering, systems, and persistence.
* __Cache invalidation on Z change__: Switching viewed/generated Z clears chunk cache to regenerate content for that slice.

## Key Entry Points (by file/function)

* `crates/core/src/resources/world.rs`
  - `World::get_tile`, `World::get_tile_cached`, `World::set_tile_cached`
  - `World::prefetch_rect`, `World::set_generation_z`
  - `World::ensure_chunk`, `World::generate_chunk`, `World::biome_for`
* `crates/core/src/save_load.rs`
  - `WorldJson`, `SaveDataJson` 3D chunk key serialization
* `crates/core/src/resources/fluids.rs`
  - `FluidManager::{process_fluids, process_fluids_clamped, seed_lithic_lava_for_chunk}`

## Testing/Verification

* Unit/integration tests round-trip `SaveDataJson` and `SaveData` and validate tile mutations persist across load, including Z.
* Client smoke tests should verify:
  - Smooth prefetching while moving in X/Y and when changing `view_z`.
  - Fluids flow horizontally and vertically and respect solid tiles per Z slice.

---

For future enhancements, consider adopting a fixed 3D grid for structure/dungeon seeding independent of `CHUNK_SIZE`, with hash-based position seeding to ensure deterministic placement across chunk sizes.