# LithicRivers Rust Migration Plan

This document captures high-level goals and a recommended rearchitecture for migrating from the Python codebase (`python-old/`) to Rust (`rust-migration/`).

## High-level goals
- Preserve core gameplay concepts while improving performance, determinism, and maintainability.
- Adopt a data-oriented/ECS architecture to align with Rust’s ownership and borrowing model.
- Keep the simulation loop deterministic and decouple it from rendering.
- Maintain chunked world data and the integer-based fluid system with “settled” optimization.
- Provide robust, secure serialization with `serde` and compact saves via MessagePack.

## Target architecture
- Workspace layout
  - `crates/core`: ECS, components, resources, systems, serialization, RNG helpers.
  - `crates/client`: ratatui TUI app for terminal rendering/input; provides ASCII-style terminal view; consumes read-only state from `core`.
  - `crates/app` (optional): if needed, additional binaries or tooling (CLI, headless sim, exporters).

- ECS (hecs)
  - Entities: IDs only.
  - Components: `Position`, `Renderable` (later), `Body`, `Inventory`, etc.
  - Resources (singletons): `World` (chunks, tiles, fluids), `Rng`, `MessageLog`, `Config`, `Keymap`.
  - Systems: movement, mining, inventory/pickup, AI/NPC, fluids, logging.
  - Schedule: ordered systems per fixed-tick with custom scheduling; client rendering reads state after sim.

- Rendering (ratatui)
  - ratatui terminal client with ASCII-style interface:
    - Map view: ASCII-style map using terminal characters with colors; viewport follows player.
    - UI panels: Inventory, status, message log, and other game panels using ratatui widgets.
  - Rendering is a strictly read-only pass that consumes a ViewModel or queries immutable state from `core`.

- Determinism
  - `rand_chacha::ChaCha20Rng` as canonical RNG.
  - Position-based seeding for worldgen/structures/dungeons.
  - Stable iteration via `indexmap` or explicit sorting when order matters.
  - Single-threaded sim by default; parallelize worldgen or chunk-partitioned work carefully.

- Serialization
  - `serde` + `rmp-serde` for compact saves; `serde_json` for debug.
  - JSON saves optional for readability; MessagePack for production.

## Data-model mapping from Python
- Keep: chunked world grid; integer fluids with settled flags; message log; event-like outcomes (as queued resources); seeded worldgen.
- Change: replace class hierarchies with components; move behavior into systems; replace ad-hoc listeners with ECS-friendly queues/schedule ordering.

## Migration phases (incremental)
1. Skeleton & minimal loop (DONE)
   - Workspace, crates, deps; minimal ECS with a `Player` that moves deterministically; ratatui "hello world".
2. Core data shapes
   - Tiles/Items/Entities as enums/IDs; basic components/resources; chunked world grid resource.
3. Save/Load
   - serde structs and versioning; JSON debug mode; MessagePack production.
4. Interaction & mining
   - Input → intents resource → movement/mining systems; inventory and drops.
5. Fluids
   - Port integer fluid model; settled optimization; frontier processing per chunk.
6. NPC/AI & messaging
   - Tickable NPC components; message log system.
7. Worldgen
   - Position-based seeding; parallel generation with `rayon`; deterministic outputs.
8. Performance passes
   - Profile hotspots; consider `parking_lot`, SIMD, chunk partitioning with deterministic reductions.

## Next steps
- Build and run the scaffold: `cd rust-migration && cargo run -p lithicrivers` (press `q` to quit).
- Define the `World` resource (chunks/tiles) and a basic `Renderable` component; render a simple ASCII map in the terminal.
- Introduce a `Config` resource and CLI flags mirroring Python settings.
- Add serde save/load for the minimal state (player position, tick).
