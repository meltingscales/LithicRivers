# Tasks

## 1. Core

- [x] 1.1 The vertical conveyor block, up/down toggling, and vertical handoff in the belt step; verify `cargo test` for lifting into a chest, a stack of three, going down, a solid target holding its item, and no loss or duplication
- [x] 1.2 Recipes and conversions; verify the reachability test and conversion crafts

## 2. Client and showcase

- [x] 2.1 Up/down glyphs and art, and the look panel naming the target layer; verify the sprite gallery and the `factory-vertical-conveyor` snapshot
- [x] 2.2 A showcase example lifting to z = 1; verify the coverage and examples-run tests

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that 3D factories are buildable and readable
