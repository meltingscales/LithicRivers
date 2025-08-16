Context: I'm going to be migrating my code from `python-old/` to `rust-migration/` and would like help setting up the rust environment. Package manager, makefile, etc.

I'm going to try to migrate as much as possible from `python-old/` to `rust-migration/` myself, and I'll ask for help when I need it. I'll try to use the same file structure.

We're using ratatui for the terminal user interface. We are not using Bevy for performance reasons.

# Rust libraries breakdown

## Client / Rendering

- ratatui (recommended): Terminal user interface library for rich TUI applications.
- crossterm (via ratatui): Cross-platform terminal manipulation for input/output.
- tui-textarea (optional): Text input widgets for ratatui.

## Simulation / Data model

- hecs (recommended): Lightweight ECS for entities/components/systems without the overhead of Bevy.
- indexmap (recommended): HashMap/Set with stable iteration for determinism.
- smallvec (optional): Inline small vectors to reduce heap allocs.
- ahash (optional): Faster hash function for high-performance maps (use with care if you need strict determinism across platforms).

## Serialization / Save system

- serde (recommended): Core serialization framework.
- rmp-serde (recommended): MessagePack format for compact saves.
- serde_json (optional): Human-readable saves, debugging.
- bincode (optional): Very fast binary serialization.
- rkyv (optional, advanced): Zero-copy serialization for maximal load speed.

## RNG / Determinism

- rand (recommended): RNG traits and utilities.
- rand_chacha (recommended): Seedable deterministic RNG (ChaCha20).

## Concurrency / Performance

- rayon (recommended): Data-parallel iteration for worldgen/sim steps.
- crossbeam (optional): Channels, deques, and concurrency utilities.
- parking_lot (optional): Faster locks than std for hotspots.

## CLI / Config / Logging

- clap (recommended): CLI args and subcommands.
- toml (optional): Config files (settings, keybinds).
- tracing + tracing-subscriber (recommended): Structured logging and spans.
- thiserror (recommended): Easy custom error types.
- anyhow (recommended): Ergonomic error handling in app code.

## Testing / Bench / Profiling

- insta (optional): Snapshot tests for rendered frames or map slices.
- proptest (optional): Property-based tests for generators/sim invariants.
- criterion (optional): Micro-benchmarks for hot loops (fluids, updates).
- pprof (pprof-rs) or cargo-flamegraph (optional): Profiling.

## Compression (if you want smaller saves)

- zstd or flate2 (optional): Compress save files transparently.

## Minimal starter set (good defaults)

- Client: ratatui (terminal UI), crossterm (terminal control)
- ECS: hecs (lightweight ECS)
- Serialization: serde, rmp-serde, serde_json
- RNG: rand, rand_chacha
- Concurrency: rayon
- Utilities: indexmap, thiserror, anyhow
- Logging: tracing, tracing-subscriber
- CLI: clap

Notes for rendering modes in ratatui:
- ASCII-style TUI: Use ratatui widgets and custom drawing for map tiles with colors and symbols; viewport follows player.
- Rich terminal UI: Utilize ratatui's built-in widgets (Block, Paragraph, List, etc.) for inventory, menus, and HUD elements.

# migration todo from python-old/

- sample UI demos for my own testing and brainstorming
- Core game loop and ECS (entities, world, game state)
- Chunked world storage and procedural world generation
- Player and NPCs (movement, state, rendering)
- Tile system (tiles, fluids, drops, destructible, tile types)
- Items and inventory system
- Body/health system (body parts, wounds, repair)
- Message log and event system
- Keymap and input handling (configurable keybinds)
- Config/settings management
- Save/load system (game saves, metadata, snapshots)
- Procedural dungeon and structure generator
- UI panels and overlays (inventory, help, body, map, etc.)
- Scripting/demo/test utilities (test worlds, test dungeons)
- Tests: simulation, worldgen, serialization, rendering
- Example/demo scripts (procedural dungeons, intro, credits)
- File browser and developer tools
- structures `.lrstructure/` format