# Tasks

## 1. Core

- [x] 1.1 `Body::remove` and what a removed part becomes (its item when working, scrap when damaged), with the torso and power source locked; verify `cargo test` for each case
- [x] 1.2 `Game::install_part_into` (install or swap) and `Game::remove_part` with their messages; verify tests that a swap returns the old part and the arm quest still completes

## 2. Client

- [x] 2.1 The Body tab's selectable part list and fitting-items list, keys and mouse; verify a snapshot script and tests with both key sets and clicks
- [x] 2.2 `body.REMOVE_PART`, `body.INSTALL_PART` on the selected part, and repair opening on it; verify a script that swaps and removes a part, and a save-reload test

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check: picking, swapping and removing parts feels clear in the Body tab
