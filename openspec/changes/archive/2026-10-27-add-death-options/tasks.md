# Tasks

## 1. Core

- [x] 1.1 `QuestMarkerType::Death` on death, named in order, and removed when its wreck is empty or gone; verify `cargo test` for adding, numbering and removal
- [x] 1.2 `keep_inventory` and `keep_body` in `WorldSettings`, read by `player_died` and `respawn`; verify tests for both, including a missing arm staying missing

## 2. Client

- [x] 2.1 The marker in the world view and on the Global Map, and deleting it after a confirmation; verify a snapshot and a headless test that the wreck keeps its items
- [x] 2.2 The `keepinventory` and `keepbody` cheats; verify headless tests, and that they survive `save-reload`

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check that finding a wreck is easy
