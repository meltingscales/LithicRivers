# Tasks

## 1. Core

- [x] 1.1 The wreck: on the player's death from either path, a corpse with a `Wreck` marker (registered for saving), the player's inventory and old body at the death position; verify `cargo test` that nothing is lost or duplicated
- [x] 1.2 `PlayerState::respawn_point` (set when a new game places the player) and `Game::respawn` with the default body, the starting kit (shared with new games) and a battle delay; verify tests for the reset body, kit and position, and that factories are untouched

## 2. Client

- [x] 2.1 The system-failure modal (cause, respawn, quit), blocking other input while dead, with click targets; verify a snapshot and headless tests of both choices
- [x] 2.2 The wreck's sprite; verify the sprite gallery
- [x] 2.3 Save while dead and reload shows the screen again; verify a headless `save-reload` test

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check that dying feels clear
