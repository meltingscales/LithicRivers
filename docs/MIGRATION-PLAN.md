# LithicRivers Architecture

> **Note:** This document focuses on the technical architecture. For release planning, see [STEAM-MVP.md](STEAM-MVP.md).

## High-level Goals
- **Performance**: Optimize for smooth gameplay on target hardware
- **Maintainability**: Clean, documented code with clear separation of concerns
- **Modularity**: Components should be loosely coupled where possible
- **Determinism**: Game state must be fully deterministic for replay/TAS support

## Core Architecture
- **ECS (Entity Component System)**: Using `hecs` for game object management
- **Deterministic Simulation**: Fixed timestep game loop
- **Chunked World**: 3D grid-based world with efficient loading/unloading
- **Serialization**: `serde` with MessagePack for save games

## Target architecture
- Workspace layout
  - `crates/core`: ECS, components, resources, systems, RNG helpers, tiles, view model, structure loading.
  - `crates/client`: ratatui TUI for terminal rendering/input; consumes read-only state from `core` and loads sprites from assets.

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
  - Client includes Inventory and Body panels; Body panel has an ASCII schematic plus a parts list.

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
1. ✅ Skeleton & minimal loop — DONE
   - `crates/core` with `Game` and ECS (`hecs`), deterministic RNG; `crates/client` ratatui app with input handling.
2. ✅ Core data shapes — DONE
   - Components: `Position`, `Player`, `Glyph`, `BlocksMovement`, `Sheep`; model `Body` present.
   - Resources: `Resources` with `World`, RNG, intents, fluids, tick; view builder exists.
   - Tiles: `TileKind` enum implemented; palette keys wired to sprites.
3. ❌ Save/Load — TODO
   - Implement `serde` data structs and versioning in `core`; add client commands to save/load.
4. 🚧 Interaction & mining — PARTIAL
   - Input → `player_move_intent` → `move_player_system()` implemented. Basic mining converts trees and drops Wood; `pickup_system()` moves `DroppedItem` into `Inventory`.
5. ✅ Fluids — DONE (initial)
   - Integer fluids with `settled` optimization, thresholds/viscosity, directional spread, per-tick processing.
6. 🚧 NPC/AI & messaging — PARTIAL
   - Simple `stumbling_sheep_system()` provides deterministic wandering. Message log exists and is rendered in the client.
7. 🚧 Worldgen — PARTIAL
   - Infinite chunked world; deterministic per-chunk sprinkle; loads example structures in chunk (0,0) from `crates/client/assets/structures/*`.
8. ❌ Performance passes — TODO
   - Add profiling, consider chunk-frontier fluid processing, data layout tweaks.

## Next steps
- 🚧 Build and run the client: `just build` then `just client` (press `q` to quit).
- 🚧 Implement Rust save/load in `crates/core` using `serde` + `rmp-serde`; add keybinds in client for save/load.
- 🚧 Expand mining and items: more tile interactions, item kinds/stacking rules, and item sprites.
- 🚧 Flesh out worldgen: position-based seeding for structures/dungeons independent of chunk size; move demo placements into seeded generators.
- 🚧 Polish message log UI; surface blocked-move feedback from `Resources.last_blocked_tile`.

## Legacy Python feature audit (for parity)
This section summarizes notable features implemented in the legacy Python codebase (`python-old/`) to ensure they are represented in the Rust migration. Status legend: ✅ DONE, 🚧 PARTIAL, ❌ TODO.

- ✅ Fluids system — integer amounts with settling optimization
  - Python: `lithicrivers/game/fluids.py`
  - Rust: implemented initial integer fluids with `settled` flag and spread rules (see `crates/core`).

- 🚧 UI system and popups (asciimatics)
  - Python: `lithicrivers/ui.py` includes dialogs, help, inventory, world map, message log, and popup manager.
  - Special case: numlock detection warning via `_detect_numlock_issue()`.
  - Rust: ratatui client present with Inventory/Body panels and message log; popups and numlock warning not yet implemented.

- 🚧 Config and keybindings with platform-specific keychords
  - Python: `lithicrivers/config_manager.py`, `lithicrivers/keymap.py`, and script `scripts/generate_platform_keychords.py`.
  - Rust: input handling exists; configurable keybinds and platform keychords need design/port.

- ❌ Steam API integration (optional)
  - Python: `lithicrivers/model/steam_api.py` with availability checks and user info.
  - Rust: not implemented.

- 🚧 Procedural dungeon generation (multiple types)
  - Python: `lithicrivers/procedural_dungeon_generator.py` supports cave systems, mining shafts, crypts, labs; deterministic via seeded RNG.
  - Rust: worldgen partial; move toward seeded, position-based dungeon/structure generation.

- 🚧 Structure generation and worldgen determinism
  - Python: `lithicrivers/structure_generator.py`, `lithicrivers/worldgen.py`; position-based seeding to ensure chunk-size independence.
  - Rust: partial sprinkle/structure loading with target of fixed-grid + position-based seeds.

- 🚧 Sprite loading, validation, and caching
  - Python: `lithicrivers/sprite_loader.py` validates dimensions, scales, caches sprites.
  - Rust: `crates/client/src/sprite_loader.rs` loads `data.json` + `sprites.txt`; ensure validation and caching parity.

- 🚧 Entities, items, dropped items, and basic inventory
  - Python: `lithicrivers/game/entities.py` defines `Entity`, `DroppedItem`, `Fluid`, simple NPCs; inventory interactions and rendering.
  - Rust: basic items (e.g., Wood), `DroppedItem`, pickup and inventory systems are present; expand item kinds/stacking rules and sprites.

- 🚧 NPCs and simple AI
  - Python: `lithicrivers/game/npcs.py` with behaviors; events in `game/events.py`.
  - Rust: simple wandering system; expand behaviors and eventing/message integration.

- 🚧 Body system and UI
  - Python: `lithicrivers/model/body.py`, demos in `examples/body_page_demo.py` and tests.
  - Rust: `Body` model present; client has Body panel; continue parity for parts list and schematic.

- 🚧 Message log and feedback
  - Python: message logging integrated across systems.
  - Rust: message log exists and is rendered; add more feedback events (e.g., blocked move, mining results).

- ❌ Save/Load parity and formats
  - Python: `lithicrivers/game/game_save_manager.py` with MessagePack via `msgspec` (legacy).
  - Rust: target `serde` + `rmp-serde` for production and `serde_json` for debug with versioned schemas.

- ✅ Test infrastructure and shared fixtures (legacy)
  - Python: optimized fixtures (`test/test_fixtures.py`), body/dungeon/sprite tests, and performance-tuned worldgen tests.
  - Rust: create analogous tests incrementally as features land.
