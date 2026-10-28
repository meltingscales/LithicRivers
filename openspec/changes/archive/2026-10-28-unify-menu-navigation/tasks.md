# Tasks

## 1. Keys

- [x] 1.1 The `menu_nav` helper, and every menu, list and window moving with both the numpad and the arrow keys; verify headless tests for each with both key sets
- [x] 1.2 Windows consume Left and Right; tab lists don't; verify a test that Left in the body repair window keeps the Body tab
- [x] 1.3 Replace hard-coded navigation keys in the Quests list, the global map and the help scroll with keybinds; verify `cargo test` and the existing snapshots

## 2. Mouse

- [x] 2.1 Click targets for the body repair window's recipes and body parts; verify a click test that selects a recipe and a part
- [x] 2.2 Click targets for the Quests list, the Inventory tab's list and tutorial selection; verify click tests

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check: menus feel the same with the numpad, the arrow keys and the mouse
