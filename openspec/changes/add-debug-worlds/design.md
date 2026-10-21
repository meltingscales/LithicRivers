# Design

## Context

- **Create New World** (`app/title_menu.rs`) has fields for name, seed, Allow cheats, Developer mode and Show tutorial, stored as `WorldSettings` (core) plus `meta.json` (client).
- **Worldgen:** `GameWorld::generate_chunk` builds Perlin terrain per chunk. `Game::new` spawns sheep, a dead dog, the android and quest structures. Structures are queued and applied when a chunk is first ensured.
- **Factory blocks** are placed through `factory::place` (`add-factory-foundation`). `crafting::mining_drop` is the mining drop table (`add-crafting-tree`).

## Goals / Non-Goals

**Goals:** five reproducible world types; data-driven ore rows and showcase builds that grow with the game; no effect on Normal worlds.

**Non-Goals:**
- A world-type editor.
- Saving custom showcases.
- Benchmark tooling (the dog arena is only a scene).

## Decisions

### World type in settings
- Add `WorldType { Normal, Void, OreTest, DogArena, FactoryShowcase }` to `WorldSettings`, with a serde default of `Normal`, so existing saves and metas load.
- `GameWorld` stores the type, because chunk generation needs it.
- `Game::new_with_settings(seed, settings)` replaces the `Game::new` + set-settings pair in `start_world`.

### Generation
- **Void:**
  - `generate_chunk` fills chunks with `Air` and skips structure queueing.
  - `Game` places a 5×5 grass platform at spawn and spawns no creatures.
- **Ore test and Factory showcase:**
  - Flat grass terrain, with no trees within 32 tiles of spawn.
  - Content is placed by `Game` after the terrain.
- **Ore rows:**
  - For each tile in `TileKind::all_variants()` that has a `mining_drop`, one row of 5 tiles, 2 tiles apart, ordered by the variant order.
  - Spouts and reactor parts are factory blocks or tiles that join automatically when they get drop-table entries or a showcase hook.
- **Dog arena:**
  - 100 dogs via `spawn_utils::spawn_enemy`.
  - Rings of radius 8, 12, 16, ..., positions chosen by `new_rng(spawn)`.
  - A position is skipped (and the next one tried) when occupied or impassable, so the count is exact.

### Showcase as data
`showcase::EXAMPLES: &[Example]`, where each `Example { name, size, build: fn(&mut Game, origin) }` places blocks and gives inputs.
- **Layout:** examples are laid out left to right in order, with a gap between them.
- **Coverage test:** builds the showcase and checks that the set of placed `FactoryKind`s equals every kind (except test-only ones), naming any missing kind.
- **Starting example:** a chest with items. Belts, furnaces and later changes append theirs.

### Developer-mode gate
The World type row is in the form only while the form's Developer mode toggle is on; turning it off resets the type to Normal.

## Risks / Trade-offs

- [Showcase builds break when machines change] → Each example is built by code using the same `factory::place`; the "examples run" test catches stalled examples.
- [100 dogs slow ticks] → It's a deliberate stress scene; the dog AI cost shows up in the test time.
- [`FactoryKind` must be enumerable for the coverage test] → Derive `strum::EnumIter` on `FactoryKind`.
