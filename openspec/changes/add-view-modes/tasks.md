# Tasks

## 1. Framework

- [ ] 1.1 Add `ViewMode`, next/previous keybinds, MODES panel label, and the state-dump field; verify headless tests for cycling, wrap-around, and that the tick doesn't change
- [ ] 1.2 Add the per-cell classification hook in the world view with dimming; verify a snapshot of Enemies view with dogs highlighted and terrain dimmed

## 2. Modes

- [ ] 2.1 Power view: supply outlines, consumer colors, generator fuel marker, blinking wires; verify snapshots with a powered, a partly powered and an unpowered machine (after `add-power-grid` and `add-tree-automation`)
- [ ] 2.2 Logistics view: belt arrows and items, fill indicators, machine status; verify snapshots with a running line and a stuck harvester (after `add-conveyor-belts` and `add-tree-automation`)
- [ ] 2.3 Save the view mode in `ClientSave` with `#[serde(default)]`; verify a save-reload test

## 3. Checks

- [ ] 3.1 Document view modes in the Help panel and AGENTS.md; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check: each mode is readable at a glance in the interactive client
