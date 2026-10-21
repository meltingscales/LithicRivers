# Tasks

## 1. Core

- [ ] 1.1 The teleporter block with a battery bank charging as a consumer; verify `cargo test` for charging, a full bank drawing nothing, and short networks
- [ ] 1.2 Jumps: cost by distance, landing tile order, destination chunk generation, and refusal when short; verify tests for each, including across chunks and z-levels
- [ ] 1.3 Default names and renaming; verify tests for unique names and a save/reload

## 2. Client and showcase

- [ ] 2.1 The teleporter window (charge, destinations, cost, rename) and the arrival effect; verify snapshots and a headless jump test
- [ ] 2.2 A recipe and a showcase example with two linked teleporters; verify the reachability and showcase coverage tests

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that teleporting feels worth its cost
