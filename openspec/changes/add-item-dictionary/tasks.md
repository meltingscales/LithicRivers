# Tasks

## 1. Core

- [ ] 1.1 Item descriptions from sprite data and `crafting::used_in`; verify tests that every item has a description and that Log is used in Wooden Plank and Power Pole

## 2. Client

- [ ] 2.1 The Item Dictionary tab with list, scrolling and details; verify a snapshot of the tab with Log selected
- [ ] 2.2 Search with a keybind; verify a headless test that searching "burn" filters the list
- [ ] 2.3 Cheat console suggestions as a scrolling list; verify a snapshot of `giveitem ` suggestions and a headless test that the last item can be reached

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that the dictionary is easy to browse
