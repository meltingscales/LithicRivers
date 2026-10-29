# Tasks

## 1. ratatui 0.30

- [x] 1.1 Upgrade `ratatui` to 0.30 and `crossterm` to 0.29 in the client and core, fixing every API change; verify `just code-clippy`, `cargo test --workspace`, and that snapshots are unchanged or each change is reviewed and explained

## 2. Window frontend

- [x] 2.1 Add `ratatui-wgpu`, `winit` and `futures-lite`, the bundled fonts with their licenses, and the font coverage test over sprites, art and UI strings; verify `cargo test` (the coverage test passes and names any missing character when one is removed)
- [x] 2.2 The input translation module (keys with repeat, named keys, text characters, mouse cells, buttons, wheel and Shift); verify unit tests with synthetic events for each mapping
- [ ] 2.3 The window frontend: event loop, backend setup, step and redraw pacing, quit on close, the `--terminal` flag and the frontend choice in `main.rs`, and the terminal fallback when no GPU backend can be created; verify `cargo build`, `just client-blind scripts/agent/smoke.txt`, and snapshot tests unchanged
- [ ] 2.4 Resizing, text size steps, fullscreen, the `window.*` keybinds in `default_config.rs`, and the saved settings; verify unit tests for the grid size from pixels and for text size limits

## 3. Options, packaging and budget

- [ ] 3.1 Text size and Fullscreen rows on the Options screen, shown as unavailable in terminal mode; verify Options snapshots for window and terminal mode and a headless test that the setting is saved
- [ ] 3.2 Windows: no console window, attaching to the parent console for `--terminal` and the headless modes; Linux launcher: start the game directly; CI system packages if the build needs any; verify `just build-windows` (or the Windows CI job) and the PR checks on all three OSes
- [ ] 3.3 The `window-render` bench scenario and the glyph-cache overflow test, both offscreen and skipped without a GPU adapter; verify `just util-bench` shows the scenario on this machine and `cargo test` skips cleanly when no adapter is found
- [ ] 3.4 Docs: AGENTS.md (window by default, `--terminal`), `docs/STEAM-MVP.md` (the launching notes), the Help panel's keybind list; verify `just code-clippy`, `cargo test --workspace` and PR checks pass

## 4. User checks

- [ ] 4.1 User check: the game opens in its own window on Linux, looks right, resizes, and the text size and fullscreen keys work
- [ ] 4.2 User check: on Windows, only the game window opens, and keystrokes are no longer consumed too fast
- [ ] 4.3 User check: on a Steam Deck in Gaming Mode, the game starts and fills the screen
