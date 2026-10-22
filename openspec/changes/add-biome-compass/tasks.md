# Tasks

## 1. Core

- [ ] 1.1 `BiomeKind::ALL` with test functions, and the "every biome is findable" test; verify `cargo test`
- [ ] 1.2 `nearest_biome` with the widening ring search, without generating chunks; verify tests for found, out of range, depth-only kinds and determinism
- [ ] 1.3 The compass item, its recipe, its charge cost through `use_power`, and `QuestMarkerType::Waypoint`; verify the reachability test and charge tests

## 2. Client

- [ ] 2.1 The compass window (biome list, result, Mark it) and waypoints on the Global Map; verify snapshots and a headless test marking a result

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that finding a far biome is satisfying
