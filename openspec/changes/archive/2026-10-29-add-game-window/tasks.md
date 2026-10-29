# Tasks

## 1. ratatui 0.30

- [x] 1.1 Upgrade `ratatui` to 0.30 and `crossterm` to 0.29 in the client and core, fixing every API change; verify `just code-clippy`, `cargo test --workspace`, and that snapshots are unchanged or each change is reviewed and explained

## 2. Window frontend

- [x] 2.1 Add `ratatui-wgpu`, `winit` and `futures-lite`, the bundled fonts with their licenses, and the font coverage test over sprites, art and UI strings; verify `cargo test` (the coverage test passes and names any missing character when one is removed)
- [x] 2.2 The input translation module (keys with repeat, named keys, text characters, mouse cells, buttons, wheel and Shift); verify unit tests with synthetic events for each mapping
- [x] 2.3 The window frontend: event loop, backend setup, step and redraw pacing, quit on close, the `--terminal` flag and the frontend choice in `main.rs`, and the terminal fallback when no GPU backend can be created; verify `cargo build`, `just client-blind scripts/agent/smoke.txt`, and snapshot tests unchanged
- [x] 2.4 Resizing, text size steps, fullscreen and the saved settings in `app.ui.window`, applied by the game window each frame; verify unit tests for text size limits and that settings round-trip through the config
- [x] 2.5 The Display panel, docked (default), floating or hidden: smaller, bigger and fullscreen buttons that change `app.ui.window`; the docked strip beside the game with clicks routed to it; the floating window always-on-top where supported, closing it hides the panel, and closing the game window closes both; verify unit tests for button hit-testing, button effects, the dock split and mode cycling

## 3. Options, packaging and budget

- [x] 3.1 Text size, Fullscreen and Display (docked/floating/hidden) rows on the Options screen, shown as unavailable in terminal mode; verify Options snapshots for window and terminal mode and a headless test that the settings are saved
- [x] 3.2 Windows: no console window, attaching to the parent console for `--terminal` and the headless modes; Linux launcher: start the game directly; CI system packages if the build needs any; verify `just build-windows` (or the Windows CI job) and the PR checks on all three OSes
- [x] 3.3 The `window-render` bench scenario and the glyph-cache overflow test, both offscreen and skipped without a GPU adapter; verify `just util-bench` shows the scenario on this machine and `cargo test` skips cleanly when no adapter is found
- [x] 3.4 Docs: AGENTS.md (window by default, `--terminal`), `docs/STEAM-MVP.md` (the launching notes), the Help panel's keybind list; verify `just code-clippy`, `cargo test --workspace` and PR checks pass

## 4. User checks

- [x] 4.1 User check: the game opens in its own window on Linux, looks right, resizes, the Display buttons change text size (with no leftover text) and fullscreen, and docked, floating and hidden all work
- Windows and Steam Deck checks (formerly 4.2 and 4.3) moved to `check-windows-and-deck`, which the user will run by hand later.
