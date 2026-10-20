# Proposal

## Why

LithicRivers is an ASCII game, and that stays the main way to play. But a 3D look at the same world (the layered underground, a factory, a fight) is a great way to show the game off and to understand vertical space that the one-layer-at-a-time ASCII view hides. A simple optional viewer window gives that without touching how the game plays.

## What Changes

- **A separate 3D viewer window**, rendered with wgpu (the Rust WebGPU implementation), opened from the in-game Menu tab (and from the title screen's Options once `add-title-and-world-menu` lands). The TUI keeps running and stays the only place you play.
- **Read-only and follows the player**: the viewer shows the world around the player in 3D (tiles as blocks across several z-layers, creatures and items as simple shapes) with a camera that follows the player. The mouse orbits and zooms around the player.
- **Tooltips on hover**: hovering over a block or creature shows its name (and, for factory blocks, its state).
- **Simple animations**: creatures and the player glide between tiles instead of jumping; belt items slide. Nothing else animates.
- **Kept deliberately basic** to stay cheap to maintain: flat-colored cubes and a fixed palette, no textures, lighting beyond simple shading, shadows or models.
- **Closing the viewer or the viewer crashing never affects the game.**

## Capabilities

### New Capabilities
- `viewer-3d`: The optional 3D viewer window: opening it, what it shows, camera, tooltips, animations, and isolation from the game.

### Modified Capabilities
<!-- None -->

## Impact

- New crate `crates/viewer` (binary `lithicrivers-viewer`) with `wgpu`, `winit` and `glam`; the game client gains no graphics dependencies.
- Core: a small serializable `WorldView3d` snapshot (tiles in a box around the player across z-layers, entities, camera target).
- Client: spawns the viewer process, streams snapshots to it over stdin when the world changes, a Menu tab action and keybind to open it.
- CI builds the viewer on Linux, Windows and macOS; rendering itself is a user check (CI has no GPU).
- Release packaging adds the viewer binary.
