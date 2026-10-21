# Tasks

## 1. Core

- [x] 1.1 `factory::take` for chests, machines and generators; verify `cargo test` for each source and that a refused insert leaves the source unchanged
- [x] 1.2 Extractor and filter extractor; verify tests for chest to belt, blocked output, the filter, and no loss or duplication
- [x] 1.3 Belt router with optional filter; verify tests for an even split, skipping blocked outputs, and filtered routing
- [x] 1.4 Walkable belts that carry the player; verify tests for riding a belt, stopping at an obstacle, and other blocks staying solid

## 2. Recipes, client and showcase

- [x] 2.1 Recipes, glyphs, the filter window and help text; verify the crafting reachability test and a snapshot of the filter window
- [x] 2.2 Showcase examples, including a self-feeding tree farm; verify the coverage test and the examples-run test

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check that routing and riding belts feel right
