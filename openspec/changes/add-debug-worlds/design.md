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

## Implementation Notes

- **Settings:** `WorldType` lives in `resources.rs` next to `WorldSettings` (`#[serde(default)]`, so old saves and `meta.json` files load as Normal). `GameWorld` keeps its own copy, saved with the chunks, so chunks first generated after a load still use the world's terrain.
- **Terrain:** Void chunks are all air. Ore test and Factory showcase are flat everywhere: rock below z = 0, grass at z = 0, air above. There are no trees anywhere, not just within 32 tiles. Debug terrain gets no structures.
- **Spawn:** Void, Ore test and Factory showcase start the player at (0, 0, 0) (`debug_worlds::DEBUG_SPAWN`) with the normal starting kit and nothing else. A Dog arena is `Game::new` plus dogs, but the player is moved to the configured surface start (`DEFAULT_PLAYER_POSITION`, (25, 25, 0)) and the rings go around it. Around the bunker spawn underground, almost every ring tile is rock.
- **Dog rings:** 25 dogs per ring, radius 8, 12, 16 and so on, on tiles shuffled by `new_rng(center)`. Rings with too few free passable tiles pass their remaining dogs to the next ring.
- **Ore rows** have no labels; looking at a tile names it. Rows start 3 tiles north of spawn, 2 rows apart, 5 tiles long, in `TileKind` variant order.
- **Showcase:** `showcase::EXAMPLES` lists Chest, Belt line into a chest, and Powered tree farm (generator, pole, auto-planter, auto-harvester, belts, chest, and grown trees). They are laid out east from (-12, -7), 3 tiles apart. `FactoryKind` derives `strum::EnumIter` (the test-only variants are iterated only in test builds), and `FactoryKind::is_test_only` excludes them from the coverage check. "Examples run" compares a fingerprint of each example's area (tiles, block contents, items lying there) before and after 60 game seconds.
- **Create form:** `CreateField` rows replace the fixed `CREATE_FIELDS` list. World type sits between Developer mode and Show tutorial, and Left/Right cycle it. Because Developer mode is on by default, the rows below it moved down one, so tests and `world-list-one-world.txt` now press Down one more time to reach Create.
