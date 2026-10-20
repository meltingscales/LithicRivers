# Proposal

## Why

`crates/client/src/main.rs` is 1,987 lines: the `App` struct and its methods, menu-tab logic, the splash screens, the whole frame layout, eleven modal renderers (corpse looting, NPC selection and dialogue, hotbar assignment, block picker, action selection, cheat console) and the game loop, all in one file. Agents and humans editing one modal have to read past the rest, and changes collide. The UI panels already live in `ui/panels/`; the modals and frame layout never followed.

## What Changes

- Move the eleven modal renderers out of `main.rs` into `ui/modals/` (one file per modal family).
- Move the splash-screen rendering and the frame layout (`ui()`, message log, bottom menu, game-viewport calculation) into `ui/` modules.
- Move `App`, its methods, `MenuTab` and `SplashState` into the `app` module, re-exported at the crate root so existing `crate::App` paths keep working.
- `main.rs` keeps only module declarations, embedded assets, `main()`, CLI handling, `step()` and `run_app()`.
- No behavior change: every rendered screen stays byte-identical.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
<!-- None: pure refactor, no requirement changes. This change sets skip_specs: true. -->

## Impact

- `crates/client/src/main.rs` (shrinks to roughly 300 lines); new `crates/client/src/ui/{layout,splash}.rs`, `crates/client/src/ui/modals/*.rs`, and `crates/client/src/app/app.rs`.
- Only the client binary changes. Core and the demos (which `#[path]`-include only `sprite_loader.rs`) are unaffected; headless and MCP entry points change at most in import paths.
- Guarded by the 15 UI snapshots, the bug scripts and the full test suite.
