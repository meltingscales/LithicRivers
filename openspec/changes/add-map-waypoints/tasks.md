# Tasks

## 1. Core

- [x] 1.1 `QuestMarkerType::Waypoint`, `deletable()` (waypoints and wrecks) and adding a waypoint with the lowest free "Waypoint N" name; verify `cargo test -p lithicrivers-core` for naming, reuse of a freed number, and a save round trip

## 2. Client

- [x] 2.1 Global Map keys (`global_map.ADD_WAYPOINT`, `RENAME_WAYPOINT`, `MOVE_WAYPOINT`, `DELETE_CHAR`, `END_RENAME`) in `default_config.rs`; add, rename (with `App::typing()`), move and delete waypoints, and refuse to delete quest markers; verify headless tests for each scenario
- [x] 2.2 The list and map show waypoints (symbol and color, the name being typed, hints for the selected marker's keys); verify a snapshot script in `scripts/snapshots/` and the font-coverage test

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that managing waypoints on the Global Map feels clear
