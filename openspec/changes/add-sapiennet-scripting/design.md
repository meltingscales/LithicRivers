# Design

## Context

- **SapienNet** (`add-sapiennet`): networks of named integer signals, evaluated once per factory step in a fixed order, and enable conditions on blocks.
- **World folders** hold `world.json.gz` and `meta.json`.

## Goals / Non-Goals

**Goals:**
- Real logic in readable code, editable with an external editor.
- Sandboxed and deterministic.
- A script error never breaks the world.

**Non-Goals:**
- Moving robots or turtles (programmable robots are a later idea).
- Networking between worlds.
- Graphics from scripts.

## Decisions

### Lua runtime
- `mlua` with the Luau backend: its sandbox, instruction-count interrupt and no-io standard library fit.
- Each computer has its own Lua state.

### Running scripts
- **When:** each factory step, after sensors write and before logic blocks evaluate. Computers run in position order.
- **Inputs and outputs:** a script sees the previous step's signal totals and emits outputs for this step.
- **The budget:** 10,000 instructions per step per computer. Over budget, the script stops with "ran out of time" and its outputs are cleared.
- **Determinism:** no clock or OS randomness is exposed. `random()` is seeded by `(seed, computer position, step count)`.

### Files
- Script files are read when a world loads and when they change on disk (a file watcher in the client, polled once a second of `app.clock` in headless runs).
- The save keeps each computer's name, its persistent `state` table (serialized, limited to 4 KB) and its last error.

### The IDE
- The in-game editor edits the same file, saving on close and on Ctrl+S (keybinds in `default_config.rs`).
- Errors show the line and message from Luau.

## Risks / Trade-offs

- [A new dependency with a C build] → Luau compiles from source with `mlua`'s vendored feature; CI builds on all three platforms before this ships.
- [Scripts in plain files can be edited to cheat] → Scripts only reach SapienNet signals and enable states, nothing the player couldn't build with logic blocks.
