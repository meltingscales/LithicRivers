# Tasks

## 1. Filter

- [x] 1.1 The toggle state, `crafting.TOGGLE_CRAFTABLE_ONLY` (default `c`) and the filter in `filtered_recipes`, combined with search, with the selection kept on the same or nearest recipe; verify `cargo test` for the starting kit, crafting updating the list, and search plus craftable-only
- [x] 1.2 The title, the hint line (clickable) and the empty-list message; verify a snapshot script in `scripts/snapshots/` and a click test
- [x] 1.3 Save the setting in `ClientSave`; verify a save-reload test

## 2. Checks

- [x] 2.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 2.2 User check: the toggle is easy to find and the filtered list feels right while crafting
