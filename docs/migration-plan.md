# LithicRivers Rust Migration Plan

This document captures high-level goals and the rearchitecture for migrating from the Python codebase (`python-old/`) to Rust (workspace under `crates/`).

## High-level goals
- Preserve core gameplay concepts while improving performance, determinism, and maintainability.
- Adopt a data-oriented/ECS architecture to align with Rust’s ownership and borrowing model.
- Keep the simulation loop deterministic and decouple it from rendering.
- Maintain chunked world data and the integer-based fluid system with “settled” optimization.
- Provide robust, secure serialization with `serde` and compact saves via MessagePack.

## Target architecture
- Workspace layout
  - `crates/core`: ECS, components, resources, systems, RNG helpers, tiles, view model, structure loading.
  - `crates/client`: ratatui TUI for terminal rendering/input; consumes read-only state from `core` and loads sprites from assets.
  - `crates/app` (optional): additional binaries or tooling (CLI, headless sim, exporters).

- ECS (hecs)
  - Entities: IDs only.
  - Components: `Position`, `Renderable` (later), `Body`, `Inventory`, etc.
  - Resources (singletons): `World` (chunks, tiles, fluids), `Rng`, `MessageLog`, `Config`, `Keymap`.
  - Systems: movement, mining, inventory/pickup, AI/NPC, fluids, logging.
  - Schedule: ordered systems per fixed-tick with custom scheduling; client rendering reads state after sim.

- Rendering (ratatui + sprite assets)
  - TUI client renders colored ASCII using sprite metadata in `crates/client/assets/sprites/**/<name>.lrsprite`.
  - `crates/client/src/sprite_loader.rs` loads `data.json` + `sprites.txt` per sprite to choose a glyph and color.
  - `crates/core/src/view.rs` builds a read-only window of entity glyphs around the player; client overlays tiles/fluids/entities and prefetches chunks.

- Determinism
  - `rand_chacha::ChaCha20Rng` as canonical RNG.
  - Position-based seeding for worldgen/structures/dungeons.
  - Stable iteration via `indexmap` or explicit sorting when order matters.
  - Single-threaded sim by default; parallelize worldgen or chunk-partitioned work carefully.

- Serialization
  - Rust: Not implemented yet. Target is `serde` with `rmp-serde` (MessagePack) for production and `serde_json` for debug.
  - Python (legacy): uses `msgspec` with MessagePack containers (`SavedGameData`, `SaveMetadata`).

## Data-model mapping from Python
- Keep: chunked world grid; integer fluids with settled flags; seeded worldgen; entity glyphs; deterministic tick loop.
- Change: replace class hierarchies with ECS components/systems; event listeners become ordered systems/queues.

## Migration phases (incremental)
1. Skeleton & minimal loop — DONE
   - `crates/core` with `Game` and ECS (`hecs`), deterministic RNG; `crates/client` ratatui app with input handling.
2. Core data shapes — PARTIAL
   - Components: `Position`, `Player`, `Glyph`, `BlocksMovement`, `Sheep`; model `Body` present.
   - Resources: `Resources` with `World`, RNG, intents, fluids, tick; view builder exists.
   - Tiles: `TileKind` enum implemented; palette keys wired to sprites.
3. Save/Load — TODO
   - Implement `serde` data structs and versioning in `core`; add client commands to save/load.
4. Interaction & mining — PARTIAL
   - Input → `player_move_intent` → `move_player_system()` implemented. Mining/inventory not yet.
5. Fluids — DONE (initial)
   - Integer fluids with `settled` optimization, thresholds/viscosity, directional spread, per-tick processing.
6. NPC/AI & messaging — PARTIAL
   - Simple `stumbling_sheep_system()` provides deterministic wandering. Message log not yet.
7. Worldgen — PARTIAL
   - Infinite chunked world; deterministic per-chunk sprinkle; loads example structures in chunk (0,0) from `crates/client/assets/structures/*`.
8. Performance passes — TODO
   - Add profiling, consider chunk-frontier fluid processing, data layout tweaks.

## Next steps
- Build and run the client: `just build` then `just client` (press `q` to quit).
- Implement Rust save/load in `crates/core` using `serde` + `rmp-serde`; add keybinds in client for save/load.
- Add mining: tile interaction system, inventory/dropped items; sprite assets for items.
- Flesh out worldgen: position-based seeding for structures/dungeons independent of chunk size; move demo placements into seeded generators.
- Add message log resource and UI panel; surface blocked-move feedback from `Resources.last_blocked_tile`.
