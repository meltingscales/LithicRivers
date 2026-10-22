# Tasks

## 1. Core

- [ ] 1.1 The reveal flag (`Reveal::Teleporter`, saved), gating recipes, the Item Dictionary and placement, with a cheat; verify `cargo test` that nothing shows before and everything after
- [ ] 1.2 The 11-by-11 multiblock (core, frame, input ports) and its completeness check; verify tests for complete, incomplete and broken frames
- [ ] 1.3 Input rate tracking and the charge model; verify tests for full rates, a short input, a long shortage and opening
- [ ] 1.4 The large-factory benchmark in CI's release job; verify it runs and passes

## 2. Client

- [ ] 2.1 The item-flow window (rates, colors, charge, time left, a link to factory stats) and the frame art; verify snapshots of each color
- [ ] 2.2 A closing screen when it opens; verify a headless test with cheated inputs

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that the goal feels earned, and the flow window helps plan the factory
