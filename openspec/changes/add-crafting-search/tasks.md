# Tasks

## 1. Shared search

- [x] 1.1 `SearchBox` in `app/input/search.rs` (typing, keep, clear, erase, matching) and the `search.*` keybinds replacing `dictionary.*`; move the Item Dictionary onto it; verify `cargo test` and the `item-dictionary` snapshot (script switched to `key search.*`)

## 2. Crafting

- [x] 2.1 `filtered_recipes` and selection as a position in it; crafting, the ingredient tree and mouse rows map through it; "Showing N of M" and "No recipes match"; verify headless tests for the plank search, crafting the second shown recipe and clearing
- [x] 2.2 Snapshot `scripts/snapshots/crafting-search.txt`; switch the mining-crafting demo from `key dictionary.*` to `key search.*`

## 3. Checks

- [x] 3.1 Update `AGENTS.md` (Item Dictionary and search keys) and the help text if it names the search key
- [x] 3.2 Verify `cargo fmt`, `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check that searching recipes feels the same as searching items
