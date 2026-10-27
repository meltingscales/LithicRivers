# Tasks

## 1. Core

- [x] 1.1 `DAY_TICKS`, the 08:00 start and the phases on `GameTime`; verify `cargo test` for phases at chosen ticks and that a new world starts in the day
- [x] 1.2 Fog of war on the surface at night only, underground unchanged; verify tests at noon, midnight and underground
- [x] 1.3 The dogs' night notice radius; verify a test of a dog 20 tiles away at night and at noon
- [x] 1.4 The `set_time` cheat (forward only); verify a headless test and the bad-format message

## 2. Client

- [x] 2.1 The MODES panel clock, and dawn and dusk shading; verify snapshots at noon, dusk and midnight
- [x] 2.2 Review every changed snapshot; scripts that pass hours set the time explicitly

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that nights feel dark but playable with a torch
