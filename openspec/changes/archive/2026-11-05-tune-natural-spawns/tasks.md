# Tasks

## 1. Spawning

- [x] 1.1 Halve the natural spawn chances in `SPAWN_RULES` (dogs 8% / 15% at night, slimes 5% Shallows and 10% Deep); verify `cargo test` (the refill and cap tests still pass)
- [x] 1.2 Replace `spawn_cooldowns` and `CHUNK_COOLDOWN_TICKS` with a saved `spawned_chunks` set that a chunk joins when it spawns, and skip spent chunks; verify a test that a spent chunk never spawns again over many checks, and that the set survives a save and reload

## 2. Docs and checks

- [x] 2.1 Update the respawn note in `AGENTS.md`
- [x] 2.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 2.3 User check that the wild feels calmer and that cleared areas staying cleared makes spawner farms worth building
  - 2026-11-05: passed ("somewhat dangerous but not impossible").
