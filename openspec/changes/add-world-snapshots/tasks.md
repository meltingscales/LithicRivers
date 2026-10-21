# Tasks

## 1. Snapshot store

- [ ] 1.1 Add the per-world snapshot store (index, files, ids, head) and `App::take_snapshot`; verify unit tests for writing, reading back exactly the saved state, and ids never reused
- [ ] 1.2 Add the triggers (5 game minutes of play, and saving); verify headless tests using pass time and the Menu-tab save
- [ ] 1.3 Add the action log and summary; verify a test that a chop and a fight between snapshots appear in the later snapshot's log and summary

## 2. Branching and pruning

- [ ] 2.1 Add rollback (snapshot the present, load, set head, save); verify the A, B, C → roll back to A → D test of parents and branches
- [ ] 2.2 Add pruning to 10 plus head; verify tests for the eleventh snapshot, keeping head, files deleted from disk, and a pruned parent shown as pruned

## 3. Timeline screen

- [ ] 3.1 Add the Timeline screen, `ui.OPEN_TIMELINE`, the snapshot details and the confirmed rollback; verify snapshot scripts for a branching timeline and a snapshot's details, and a headless test that cancelling a rollback changes nothing
- [ ] 3.2 Remove the unused `SNAPSHOTS_FOLDER` setting; document the Timeline in Help and AGENTS.md; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check: snapshots, rolling back and reading the action log feel right in the interactive client
