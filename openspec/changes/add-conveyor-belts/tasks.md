# Tasks

## 1. Core

- [x] 1.1 Add `FactoryKind::Belt`, `BeltSlot`, and the item handoff interface (implemented for chests); verify unit tests for `accepts` / `insert` on a chest
- [x] 1.2 Add the belt system with sink-first ordering; verify `TestWorld` tests for a straight line, a corner, two lines merging, a loop, a full chest backing up the line, and item-count conservation
- [x] 1.3 Belts pick up dropped items; verify a test
- [x] 1.4 Verify belt lines across x = 63/64 and x = -1/0 deliver in the same number of steps as within one chunk

## 2. Recipe, client and checks

- [x] 2.1 Add the wooden conveyor recipe; verify the crafting reachability test still passes
- [x] 2.2 Belt glyphs with carried items; dropping onto an adjacent belt; verify a headless script building a 4-belt line into a chest, dropping logs on it, passing time, and checking the chest; add a snapshot of the running line
- [ ] 2.3 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that belts read clearly
