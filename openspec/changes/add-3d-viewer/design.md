# Design

## Context

- The client's main thread runs the ratatui terminal loop. `winit` requires its event loop on the main thread on macOS, so a viewer window can't live on a background thread of the client process.
- There is no pause/options menu yet; the in-game Menu tab handles save/load, and `add-title-and-world-menu` plans an Options screen.
- `build_render_view` produces ASCII lines for the TUI; there's no 3D-friendly snapshot.

## Goals / Non-Goals

**Goals:** an optional, read-only 3D window that's cheap to maintain and can't break the game.

**Non-Goals:** playing from the viewer (input to the game), textures, models, lighting beyond simple shading, shadows, particles, the browser / WebAssembly, VR, recording.

## Decisions

### Separate process, separate crate
A new workspace crate `crates/viewer` builds `lithicrivers-viewer`. The client spawns it as a child process and writes snapshots to its stdin. A separate process sidesteps the main-thread constraint, keeps `wgpu`/`winit` out of the game binary, and means a viewer crash can't take the game down. The child is killed when the game exits.
Alternative: in-process window on a second thread. Rejected: impossible on macOS and couples the game to GPU drivers.

### Snapshot protocol
Core gains `WorldView3d { tick, player: Pos, entities: Vec<EntityView>, tiles: Vec<(Pos, TileKind)> }` for a box of 48×48 tiles and ±4 z-layers around the player (air omitted), plus factory-block state for tooltips. The client sends it as length-prefixed JSON whenever the game tick or player position changes (not every frame), and on open. Version field for compatibility. Building the snapshot is a pure read of the game, so determinism is unaffected (tested by running scripts with snapshot building enabled vs disabled).

### Rendering
`wgpu` with one instanced cube pipeline: per-instance position and color from a fixed tile/entity palette, simple directional shading, depth test. Entities are smaller cubes; items are tiny cubes. The player's layer and above are cut away within a radius around the player so the player is visible underground. Camera: orbit around the player target (mouse drag), zoom (wheel), smooth follow (exponential lerp toward the new target).

### Tooltips
CPU ray pick against the instance grid (voxel DDA from the cursor ray), then draw the label as a screen-space text quad (`glyphon` or a tiny bitmap font). Keep to one line plus optional state.

### Animations
The viewer keeps each entity's previous position and lerps over 150 ms when a snapshot moves it; belt items lerp along their belt direction. Entities are matched by entity ID across snapshots.

### Opening and errors
Menu tab action plus `ui.OPEN_3D_VIEWER` (default `3`). If the binary is missing or exits immediately, the client logs "3D viewer couldn't start: <reason>". In headless runs and MCP, the viewer is never spawned.

### CI and packaging
PR checks build the viewer crate on all three OSes (Linux needs no extra system packages for `wgpu`/`winit` beyond the X11/Wayland headers `scripts/install_dev_deps.sh` already installs). Tests cover the snapshot builder and protocol; rendering is a user check. Release staging copies `lithicrivers-viewer` next to the client.

## Risks / Trade-offs

- [wgpu/winit upgrades are churny] → Pin versions; the viewer is isolated in its own crate and can lag behind.
- [Build time and CI time grow] → Separate crate; the cached release build absorbs most of it after the first run.
- [Steam / macOS launch quirks for a second binary] → Release packaging task; the viewer is optional, so the game ships even if it fails.
