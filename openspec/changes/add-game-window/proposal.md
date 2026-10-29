# Proposal

## Why

The game only runs inside a terminal emulator, so what a Steam player sees depends on their machine. On Linux, `scripts/launcher/lithicrivers-launcher.sh` opens whatever terminal it finds first (`x-terminal-emulator`), which can come up tiny with a poor font. Steam Deck's Gaming Mode has no terminal emulator at all, so the game likely can't show anything there. On Windows the console window has the same font and size problem, and is the likely cause of keystrokes being consumed too fast. A window the game owns, drawn with a font we ship, looks the same everywhere. Controller support and the Steam overlay both need one too.

## What Changes

- **Upgrade ratatui** from 0.26 to 0.30 and **crossterm** from 0.27 to 0.29, in the client and core. Every window backend needs ratatui 0.30. This is a prerequisite with no visible change, and snapshots must stay the same or be reviewed.
- **A game window by default.** The client opens its own window (winit) and draws the same ratatui screen on the GPU with `ratatui-wgpu`, using a bundled monospace font (DejaVu Sans Mono). The user decided: option B, ratatui-wgpu on winit rather than egui.
- **Input unchanged.** Window key presses become the same crossterm `KeyCode`s, and mouse clicks, scrolls and moves become screen cells, so keybinds, menus and mouse handling work as before.
- **Resizing and text size.**
  - Resizing the window recomputes the columns and rows.
  - A **Display panel** has buttons to make the text bigger or smaller and toggle fullscreen. It's docked along the game window's edge by default, or floats as a small separate window, or is hidden (decided by the user).
  - Options gains Text size, Fullscreen and Display rows, saved to the config file.
- **Terminal mode kept:** `--terminal` runs the game in the current terminal exactly as today, for players who prefer it.
- **Launching:**
  - Windows builds open no console window, unless started with `--terminal`.
  - The Linux launcher stops searching for a terminal emulator and just starts the game.
- **Font coverage:** a test checks that every character the game can draw (sprites, UI, art) exists in the bundled fonts.
- **Performance:** `just util-bench` gains a scenario that draws through the GPU backend, so the 16 ms frame budget covers the window too.
- Headless runs, MCP and snapshots keep using the in-memory `TestBackend` and don't change.

## Capabilities

### New Capabilities
- `game-window`: the game's own window: rendering with the bundled fonts, input translation, resizing, text size, fullscreen, and terminal mode as the alternative.

### Modified Capabilities
- `headless-client`: "Interactive client unchanged" now says the interactive client opens the game window by default, with `--terminal` for the old terminal UI.
- `title-menu`: the Options screen gains Text size and Fullscreen.

## Impact

- **Dependencies:** `ratatui` 0.30, `crossterm` 0.29, `ratatui-wgpu` 0.6 (wgpu 30), `winit` 0.30, and a blocking executor for the backend's async builder (`futures-lite`, as its examples use). This is the main dependency cost. It replaces the terminal emulator as a runtime requirement.
- **Assets:** one font file under `crates/client/assets/fonts/` with its license.
- **Client:** `main.rs` (frontend choice), a new window module (event loop for the game window, the docked panel and the floating Display window, input translation, backend setup), every ratatui 0.30 API change across the UI, `default_config.rs` settings, the Options screen.
- **Core:** the crossterm bump in `keycode_mapping.rs`.
- **Packaging:** the `windows_subsystem` attribute, the launcher script, and CI system packages for winit and wgpu on Linux if the build needs any.
- **Docs:** AGENTS.md (window versus `--terminal`), `docs/STEAM-MVP.md` (the launching section).
