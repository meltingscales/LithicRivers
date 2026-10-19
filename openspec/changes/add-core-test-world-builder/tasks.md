# Tasks

## 1. Refactor game construction

- [x] 1.1 Add a golden test capturing entity kinds and positions from `Game::new(12345)` after its first tick; verify it passes on the current code
- [x] 1.2 Extract sheep and NPC spawn tuples from `Game::new` into `spawn_utils` functions; verify the golden test still passes
- [x] 1.3 Split `Game::new` into `Game::new_empty(seed, player_pos)` plus default spawns; verify the golden test still passes and `new_empty` yields only the player

## 2. Builder

- [x] 2.1 Add `test_support` module behind `cfg(any(test, feature = "test-support"))` and the `test-support` feature; verify `cargo build --release -p lithicrivers-client` does not compile it (e.g. the symbol is absent with the feature off)
- [x] 2.2 Implement `TestWorld` construction: seed, player position, inventory, tile/fill, sheep, feral dog, NPC, dropped item, with handles; verify tests for the empty scenario, floor fill, and dog placement specs
- [x] 2.3 Implement `TestGame` drive and query helpers (`act`, `tick`, `ticks`, `pos`, `health`, `player_inventory`); verify the chop-tree scenario test and the same-fixture-twice determinism test pass

## 3. First builder-based tests

- [x] 3.1 Port `test_tree_chop_drops_are_deterministic` in `player_movement.rs` to the builder; verify it passes with less setup code
- [x] 3.2 Add combat-entry tests (player moves into a feral dog, combat state becomes active; dog at zero health ends combat); verify they pass
- [x] 3.3 Add item pickup/interaction tests (auto-pickup on, walk onto a dropped item, item lands in inventory); verify they pass
- [x] 3.4 Mention the builder in AGENTS.md's Testing section with a short example; verify the example compiles as a doctest or matches a real test
