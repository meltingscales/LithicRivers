# Tasks

## 1. Filter

- [ ] 1.1 The toggle state, `crafting.TOGGLE_CRAFTABLE_ONLY` (default `c`) and the filter in `filtered_recipes`, combined with search, with the selection kept on the same or nearest recipe; verify `cargo test` for the starting kit, crafting updating the list, and search plus craftable-only
- [ ] 1.2 The title, the hint line (clickable) and the empty-list message; verify a snapshot script in `scripts/snapshots/` and a click test
- [ ] 1.3 Save the setting in `ClientSave`; verify a save-reload test

## 2. Checks

- [ ] 2.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 2.2 User check: the toggle is easy to find and the filtered list feels right while crafting
