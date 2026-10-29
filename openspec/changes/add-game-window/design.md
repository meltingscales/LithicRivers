# Design

## Context

- `main.rs` sets up crossterm (raw mode, alternate screen, mouse capture), builds `Terminal<CrosstermBackend>` and runs `run_app`. That loop calls `step(app)`, draws with `ui(f, app)`, waits up to 50 ms for an event, and passes key presses to `handle_input(app, KeyCode)` and mouse events to `mouse_input` → `handle_mouse` (a cell, a button, Shift).
- The headless runner reproduces the same loop over `TestBackend`, which is what snapshots, MCP and `--bench` use.
- 48 client files and core's `keycode_mapping.rs` use crossterm's `KeyCode`, and keybinds map config names to `KeyCode`s.
- `ratatui-wgpu` 0.6 builds a `WgpuBackend` from fonts, a pixel size and a target: a winit `Window`, or an offscreen texture (`RenderTexture`). It takes pixel sizes on resize and supports font fallback. Its glyph cache holds about 3,750 unique character and style combinations per draw. Its examples drive it from a winit 0.30 `ApplicationHandler` and block on its async builder.

## Goals / Non-Goals

**Goals:**
- One rendering and input path for the window, written once next to the terminal path, with the game code (`step`, `ui`, `handle_input`, `handle_mouse`) unchanged.
- A look that's identical on Linux, Windows and the Steam Deck.
- Keep `--terminal`, headless runs and snapshots working exactly as today.

**Non-Goals:**
- Controller or Steam Input support, and Steamworks or overlay work. They come later and build on the window.
- CRT or scanline shaders. The `PostProcessor` hook makes them easy later, but the default look stays flat.
- Changing any panel's layout for the window. It draws what the terminal draws.
- A web build.

## Decisions

**ratatui-wgpu on winit (decided by the user).** It draws on the GPU, which fits the Steam Deck's CPU and the 16 ms frame budget, and it has a small dependency tree. We considered `egui_ratatui` on eframe: less window code for us, but CPU rasterizing every frame plus a whole UI toolkit. The cost of this choice is a winit event loop we own, kept to one module.

**The ratatui 0.30 upgrade lands first, by itself.** It's needed for any backend, and it touches the whole UI (renamed `Frame` and buffer APIs, the `unstable-rendered-line-info` feature). A separate first step keeps the window work reviewable, and the snapshot suite tells us whether rendering changed. Any intended snapshot change is reviewed and explained, never accepted blindly.

**Translate input at the edge.** A `window::input` module turns winit events into what the game already takes:
- **Keys:** `KeyEvent` on press (including key repeat, so holding a direction keeps moving) becomes a `KeyCode`. Named keys (Enter, Escape, Tab, BackTab, arrows, Home/End, PageUp/PageDown, Insert/Delete, Backspace, F1–F12) map one to one. Everything else uses the event's text, so Shift gives the same characters as the terminal.
- **Mouse:** the pointer's pixel position is divided by the cell size from the font metrics to give a cell. Button presses, wheel steps (Shift included) and moves become `MouseInput`.

Pure functions are covered by unit tests with synthetic events, so the mapping is tested without a window. The alternative, teaching the game a second event type, would touch every input file for no gain.

**Two frontends behind one loop body.** `run_app` keeps the terminal frontend. The window frontend is an `ApplicationHandler`:
- `resumed` creates the window and backend.
- Input events are handled when they arrive.
- `about_to_wait` runs `step` and asks for a redraw, with `ControlFlow::WaitUntil` about 16 ms ahead (60 frames a second at most). The terminal's 50 ms poll capped it at 20.
- `RedrawRequested` draws with `ui`.
- `CloseRequested` and the game's quit flag exit the loop.

Combat auto-ticks and splash timers already read `app.clock`, so frame pacing doesn't change game timing.

**Grid from pixels.** On resize or a text-size change, the backend is resized in pixels and ratatui's `Terminal` re-reads its size, so columns are the width divided by the cell width, and rows likewise. Text size is a font pixel height in steps (for example 12 to 32, default 18), stored under `settings.ui.WINDOW_FONT_PX` with `WINDOW_FULLSCREEN`. Keybinds go in a new `window` category in `default_config.rs` (`TEXT_BIGGER`, `TEXT_SMALLER`, `TOGGLE_FULLSCREEN`), checked with `keybinds.matches`. Changing the font size rebuilds the backend's fonts.

**Fonts: DejaVu Sans Mono plus GNU Unifont as fallback.** DejaVu Sans Mono covers box drawing, block elements and most symbols, under a permissive license. Unifont covers the rest of the Basic Multilingual Plane, under the SIL OFL. Both ship in `crates/client/assets/fonts/` with their license files and are embedded with `include_bytes!`. A coverage test gathers every character from:
- the sprite files and art assets
- the string literals the UI draws: a scan of the client's `ui/` sources and the dialogue database, together a good approximation
- the box and border sets

It then checks each one against the two fonts' character maps (read with the shaping crate's face parser), and fails naming any it doesn't find. Emoji outside the BMP would fail it. That's deliberate: the art direction already rules them out.

**Offscreen rendering for tests and the benchmark.** The bench gains `window-render`: the surface-walk script drawn into a `RenderTexture` of 80x24 cells instead of `TestBackend`. If no GPU adapter is available, as on CI runners, the scenario is reported as skipped, not failed. The glyph cache limit is checked the same way: a test draws the busiest screens offscreen (the factory showcase, inventory, Item Dictionary) and fails if the cache overflows. It's skipped without an adapter.

**Windows: no console.** `#![cfg_attr(windows, windows_subsystem = "windows")]` on the client binary. With `--terminal`, `--script`, `--bench` or `--mcp`, it calls `AttachConsole(ATTACH_PARENT_PROCESS)` (winapi is already a Windows dependency) so terminal output still works from a console.

**Linux launcher.** The script keeps its log file, but runs the binary directly instead of searching for a terminal emulator. `--terminal` passes through for players who want it.

## Risks / Trade-offs

- [The ratatui upgrade changes rendering details] → Snapshot diffs are read cell by cell. Intended changes are accepted with a reason in the PR, and anything else is fixed before the window work starts.
- [Glyph cache overflow on busy screens] → The offscreen overflow test covers the busiest screens. If it overflows, lower color variety in that view or raise the cache through the backend's options.
- [No GPU, or a broken driver] → If the backend can't be created, the game logs why and falls back to `--terminal` when started from a terminal. Otherwise it shows a message box explaining how to start it with `--terminal`.
- [Young dependency (ratatui-wgpu 0.x)] → It's used only in the window module, behind ratatui's `Backend` trait, so swapping to another backend later touches one file.
- [I can't open a window here] → Everything that can be tested without a window is: input mapping, coverage, offscreen rendering, snapshots. Launching, fullscreen, the Deck and the feel are user checks.

## Migration Plan

The window becomes the default in the same release that ships it. `--terminal` keeps the old behavior, and the config gains two settings with defaults, so old configs keep working. Rolling back means making `--terminal` the default again.
