# Tasks

## 1. Core

- [x] 1.1 `Body::effects()` and `BodyEffects` with the table's constants; verify `cargo test` for every row, Enhanced parts, and today's damaged-leg walking unchanged
- [x] 1.2 Mining and breaking costs from the arms, and no mining without arms; verify tests that a damaged leg doesn't slow mining and a missing arm does
- [x] 1.3 Melee damage from the arms, and no melee without arms (tackle still works); verify combat tests and a deterministic replay
- [x] 1.4 The light radius from the head, and energy regen from the power source; verify tests with and without a torch

## 2. Client

- [x] 2.1 The Body tab's effects list and the context-help warning; verify snapshots of the starting body and a repaired one
- [x] 2.2 Review every changed snapshot and script timing

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that damage is felt but not frustrating, especially the starting body
