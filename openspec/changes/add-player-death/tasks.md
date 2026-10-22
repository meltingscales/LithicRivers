# Tasks

## 1. Core

- [ ] 1.1 The wreck: a corpse with a `Wreck` marker and the player's inventory at the death position (registered for saving); verify `cargo test` that nothing is lost or duplicated
- [ ] 1.2 `PlayerState::backup_point` (defaulting to the bunker spawn) and `Game::boot_backup_body` sharing the new-game body and kit code; verify tests for the reset body, kit and position, and that factories and quests are untouched
- [ ] 1.3 A short battle delay for creatures near the backup point; verify a test that a dog next to it doesn't attack at once

## 2. Client

- [ ] 2.1 The system-failure modal (cause, boot, quit), blocking other input while dead, with click targets; verify a snapshot and a headless test of both choices
- [ ] 2.2 The wreck's sprite; verify the sprite gallery
- [ ] 2.3 Save while dead and reload shows the screen again; verify a headless `save-reload` test

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that dying feels clear and fair
