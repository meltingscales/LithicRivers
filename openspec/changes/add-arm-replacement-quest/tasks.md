# Tasks

## 1. Core

- [ ] 1.1 `ItemKind::SalvagedArm`, and `Body::install` with its part table; verify `cargo test` for installing into a missing arm, refusing a working one, and repairs working afterwards
- [ ] 1.2 The quest-treasure tile spawning the arm chest once per world (`quest_chests_placed`, saved, with the old-save fallback); verify tests for one chest after re-entering and a save from before this change
- [ ] 1.3 `QuestType::ReplaceArm`, the `InstallPart` objective, the android's offer after the repair quest, and in-order completion; verify a test of the full flow with save and reload between steps

## 2. Content and client

- [ ] 2.1 Place the `1` tile in the facility's assembly-line level, past its enemies; verify a test that the chest is reachable from the facility's entrance
- [ ] 2.2 The Body tab's Install action and sprites for the arm and the quest chest; verify a snapshot and the sprite gallery
- [ ] 2.3 A headless script that plays the quest from the android to the install (using teleport cheats for the walk); verify it in `scripts/snapshots/`

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that the quest reads well and the walk to the facility is worth it
