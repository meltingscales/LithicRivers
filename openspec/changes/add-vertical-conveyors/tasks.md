# Tasks

## 1. Core

- [x] 1.1 The vertical conveyor block, up/down toggling, and vertical handoff in the belt step; verify `cargo test` for lifting into a chest, a stack of three, going down, a solid target holding its item, and no loss or duplication
- [x] 1.2 Recipes and conversions; verify the reachability test and conversion crafts

## 2. Client and showcase

- [x] 2.1 Up/down glyphs and art, and the look panel naming the target layer; verify the sprite gallery and the `factory-vertical-conveyor` snapshot
- [x] 2.2 A showcase example lifting to z = 1; verify the coverage and examples-run tests

## 3. Climbing and stairs (added during implementation)

- [x] 3.1 Climbing vertical conveyors like stairs; verify `cargo test` for climbing a stack up and down
- [x] 3.2 The Stairs item, recipe, stairway placement (up or down, digging through rock) and breaking; verify `cargo test` for digging up, going down, too few items, and breaking one back
- [x] 3.3 Ghost stairs on tiles with something to climb above or below; verify the `stairs-ghost` snapshot
- [x] 3.4 A "Between layers" demo scene; verify the demo scenario test and its snapshot
- [x] 3.5 Moving up or down with nothing to climb offers to build a stairway where the player stands (Yes/No), or says to craft stairs and try again; verify the `stairs-prompt` snapshot and tests for No, clicking Yes, and a climbable stack not asking

## 4. Checks

- [x] 4.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.2 User check that 3D factories are buildable and readable, and that stairs and their ghosts read clearly
