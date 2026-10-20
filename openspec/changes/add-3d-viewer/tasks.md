# Tasks

## 1. Snapshot

- [ ] 1.1 Add `WorldView3d` and its builder in core (48×48 box, ±4 layers, entities, factory state) with a versioned length-prefixed JSON encoding; verify unit tests for contents, size bounds and round-trip encoding, and a test that running the smoke script with the builder enabled leaves game state identical

## 2. Viewer

- [ ] 2.1 Create `crates/viewer` (`wgpu`, `winit`, `glam`, pinned) reading snapshots from stdin and drawing instanced cubes with a palette, following and orbiting the player; verify it builds on all three OSes in PR checks
- [ ] 2.2 Add hover picking and tooltips; verify a unit test for the voxel ray pick
- [ ] 2.3 Add movement and belt-item interpolation by entity ID; verify a unit test of the interpolation math

## 3. Client

- [ ] 3.1 Spawn and feed the viewer from the Menu tab action and `ui.OPEN_3D_VIEWER`; bring an existing window forward instead of spawning twice; log a message if it can't start; never spawn in headless/MCP; verify headless tests that opening the viewer in a headless run is a no-op with a message, and that a missing viewer binary logs the failure
- [ ] 3.2 Add the viewer binary to release staging; verify `just stage-artifacts-release-linux` includes it

## 4. Checks

- [ ] 4.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.2 User check: open the viewer in the interactive client, walk around above ground and in the bunker, hover a dog and a chest, and close it mid-game
