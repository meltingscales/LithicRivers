# Tasks

## 1. Core

- [ ] 1.1 Entrance and exit kinds, linking on placement within 10 tiles, and the link saved; verify `cargo test` for linking, out of range, re-linking after a break, and save/reload
- [ ] 1.2 Transit in the belt step and belt order, with backing up; verify tests for crossing a belt line, the same rate as belts, a full exit, and no loss or duplication
- [ ] 1.3 Breaking either end; verify tests for items in transit dropping on the entrance's tile
- [ ] 1.4 Recipe and blueprint support (both ends re-link on paste); verify the reachability test and a blueprint test

## 2. Client and showcase

- [ ] 2.1 Sprites, the blinking tunnel line and the look panel naming the other end; verify the sprite gallery and a snapshot
- [ ] 2.2 A showcase example of two crossing lines; verify the coverage and examples-run tests

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that crossing lines is easy to build and to read
