# Tasks

## 1. Core

- [ ] 1.1 `WorldSettings::tas_mode` (saved, fixed after creation) and a state hash over the core save data; verify `cargo test` (the hash is stable across save-reload and differs after a move)

## 2. Client

- [ ] 2.1 Combat auto-ticks and other clock-driven behavior read `app.clock`, never `Instant::now()`; verify `cargo test` (snapshots unchanged) and a check that `Instant::now()` only appears in the clock
- [ ] 2.2 TAS clock: in a TAS world the clock moves only on `tas.FRAME_ADVANCE`; keybinds in `default_config.rs`; verify a headless test that combat doesn't advance without frame advances
- [ ] 2.3 The input recorder writing script steps to `<world>/tas/movie.txt`, shared with MCP `export_script`; verify a test that records a run and replays it with the headless runner to the same state hash
- [ ] 2.4 Savestate slots and the rerecord count; verify a headless test of save, play, load
- [ ] 2.5 TAS mode on Create New World, the world-list mark, Play movie and the TAS HUD; verify snapshots in `scripts/snapshots/`
- [ ] 2.6 `expect-state-hash` script step; verify a headless test with a matching and a mismatched hash

## 3. Docs and checks

- [ ] 3.1 `docs/TAS.md` and the script-step table in `AGENTS.md`
- [ ] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check that frame advance and savestates feel right in the live client
