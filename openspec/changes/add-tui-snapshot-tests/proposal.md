# Proposal

## Why

UI regressions (a panel overflowing, a modal not closing, a message not showing) are only caught today when someone plays the game. Because the game renders ASCII, a rendered screen is plain text that can be stored and diffed in `cargo test`. Once `add-headless-client-driver` lands, the text is already available.

## What Changes

- Add snapshot tests that run headless scripts from `scripts/snapshots/` and compare their full stdout (screen and state dumps) against stored snapshot files.
- Seed the suite with scripts covering the main UI: world view after start, each menu tab, inventory, crafting, help, combat panel, cheat console, and the corpse-loot and NPC dialogue modals where reachable.
- Snapshot mismatches fail `cargo test` with a readable text diff; a documented command accepts intended changes.
- Add `insta` as a client dev-dependency.

## Capabilities

### New Capabilities
- `tui-render-snapshots`: Stored text snapshots of scripted client runs that `cargo test` checks for unintended UI or state changes.

### Modified Capabilities
<!-- None: no existing specs. -->

## Impact

- Depends on `add-headless-client-driver` (script runner, screen/state dumps, determinism).
- `crates/client/Cargo.toml`: `insta` dev-dependency.
- New test module in the client crate; new `scripts/snapshots/*.txt` and `crates/client/src/snapshots/*.snap` files.
- AGENTS.md: how to review and accept snapshot changes.
- Any intentional UI or worldgen change will require updating snapshots in the same PR.
