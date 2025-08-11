Awesome—here’s a simple, focused list of Rust dependencies and what you’ll use them for. I’ve grouped them and marked a minimal recommended set vs optional.

Core TUI

ratatui (recommended): TUI layout, widgets, and rendering.
crossterm (recommended): Terminal backend, input, and event handling.
Simulation / Data model

hecs or shipyard or bevy_ecs (pick one):
hecs (recommended): Lightweight, fast ECS for entities/components.
shipyard: Parallel-friendly ECS with good ergonomics.
bevy_ecs: Feature-rich ECS extracted from Bevy.
indexmap (recommended): HashMap/Set with stable iteration for determinism.
smallvec (optional): Inline small vectors to reduce heap allocs.
ahash (optional): Faster hash function for high-performance maps (use with care if you need strict determinism across platforms).
Serialization / Save system

serde (recommended): Core serialization framework.
rmp-serde (recommended): MessagePack format for compact saves.
serde_json (optional): Human-readable saves, debugging.
bincode (optional): Very fast binary serialization.
rkyv (optional, advanced): Zero-copy serialization for maximal load speed.
RNG / Determinism

rand (recommended): RNG traits and utilities.
rand_chacha (recommended): Seedable deterministic RNG (ChaCha20).
Concurrency / Performance

rayon (recommended): Data-parallel iteration for worldgen/sim steps.
crossbeam (optional): Channels, deques, and concurrency utilities.
parking_lot (optional): Faster locks than std for hotspots.
CLI / Config / Logging

clap (recommended): CLI args and subcommands.
toml (optional): Config files (settings, keybinds).
tracing + tracing-subscriber (recommended): Structured logging and spans.
thiserror (recommended): Easy custom error types.
anyhow (recommended): Ergonomic error handling in app code.
Testing / Bench / Profiling

insta (optional): Snapshot tests for rendered frames or map slices.
proptest (optional): Property-based tests for generators/sim invariants.
criterion (optional): Micro-benchmarks for hot loops (fluids, updates).
pprof (pprof-rs) or cargo-flamegraph (optional): Profiling.
Compression (if you want smaller saves)

zstd or flate2 (optional): Compress save files transparently.
Minimal starter set (good defaults)

TUI: ratatui, crossterm
ECS: hecs
Serialization: serde, rmp-serde, serde_json
RNG: rand, rand_chacha
Concurrency: rayon
Utilities: indexmap, thiserror, anyhow
Logging: tracing, tracing-subscriber
CLI: clap