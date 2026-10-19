# Design

## Context

- `Game::new(seed)` (`crates/core/src/game.rs`) spawns the player plus sheep, a dead feral dog, NPCs and queues structures; tests using it inherit that noise.
- Tiles come from seeded worldgen on demand (`get_tile_cached`); `set_tile_cached` overrides a tile.
- Spawn logic lives in several places: inline `world.spawn((...))` in `Game::new`, and `spawn_utils::world_spawn_feraldog`.
- Player actions are `PlayerIntent`s on `res.player_state.intent`; `Game::tick()` returns `GameTickResult` bitflags.
- The client has no lib target, so shared test code must come from core.

## Goals / Non-Goals

**Goals:**
- A test describes only what matters to it.
- Reuse the real spawn code so fixtures can't drift from gameplay entities.

**Non-Goals:**
- Changing what `Game::new` spawns for real play.
- A scenario file format (agents can use headless scripts for that).

## Decisions

### Split player creation out of `Game::new`
Refactor `Game::new` into `Game::new_empty(seed, player_pos)` (resources + player only) followed by the existing default spawns. `Game::new` keeps its behavior; the builder starts from `new_empty`. This puts player construction in one place instead of duplicating its component list in test code.

### Extract shared spawn functions
Move the inline sheep and NPC spawn tuples from `Game::new` into `spawn_utils` functions (feral dog already has one). `Game::new` and the builder both call them. Behavior-preserving refactor; covered by an equality test that `Game::new` produces the same entities before and after.

### Builder shape
`TestWorld::new(seed).player_at(x,y,z).inventory(&[..]).auto_pickup(bool).fill(a, b, TileKind).tile(p, TileKind).sheep(p).feral_dog(p).npc(p, ..).item(p, ItemKind, qty).build() -> (TestGame, Handles)`. `TestGame` wraps `Game` with `act(PlayerAction)`, `tick()`, `ticks(n)`, and query helpers (`pos(handle)`, `health(handle)`, `player_inventory()`). Entity adds return handles immediately (an index resolved after build), so tests don't search by position.

### Placement order
Tiles are applied after worldgen for the touched chunks is forced (`ensure_chunk`), so later lazy generation can't overwrite them.

### Visibility
Module `test_support` under `#[cfg(any(test, feature = "test-support"))]`. Client adds `lithicrivers-core = { path = "../core", features = ["test-support"] }` under `[dev-dependencies]` only when snapshot tests need it.

## Risks / Trade-offs

- [Refactoring `Game::new` changes startup state] → Golden test comparing entity kinds/positions from `Game::new(12345)` before and after the refactor.
- [Worldgen runs as ticks generate chunks around the player] → `new_empty` queues no quest structures, but ticking still runs real chunk generation, which can place structure loot (dropped items) in nearby chunks (e.g. two items at (42,22) for seed 1). No creatures or NPCs appear; a test asserts this. Tests look things up by handle or position rather than counting all entities.
- [Builder grows into a second game API] → Keep it to construction + thin drive/query helpers; anything else goes through `Game`.
