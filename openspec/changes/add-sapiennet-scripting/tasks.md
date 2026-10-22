# Tasks

## 1. Core

- [ ] 1.1 `mlua` with Luau (vendored), one sandboxed state per computer, and the instruction budget; verify `cargo test` for the budget, no io, and CI builds on all platforms
- [ ] 1.2 The API (`signals`, `emit`, `enable`, `print`, seeded `random`) and per-step execution order; verify tests for each call and determinism
- [ ] 1.3 Script files in the world's `scripts/` folder, the saved `state` table and last error; verify tests for loading, reload on change and save/reload

## 2. Client

- [ ] 2.1 The IDE window (editor, line numbers, syntax colors, signals, errors, log) with its keybinds; verify snapshots and a headless test editing and saving a script
- [ ] 2.2 Watching the scripts folder (polled on `app.clock` in headless runs); verify a headless test that an outside edit is picked up

## 3. Docs and checks

- [ ] 3.1 `docs/SAPIENNET-SCRIPTING.md` with the API and example scripts, and examples on dead robots in the world; verify the examples run in tests
- [ ] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check: write a farm script in an outside editor and watch it run
