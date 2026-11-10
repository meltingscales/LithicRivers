# Tasks

## 1. Core

- [x] 1.1 Power history ring buffers per network id, with merge and expiry; verify `cargo test` for sampling once per second, merging and expiry
- [x] 1.2 Item flow counters and `stats::record` at every place items are created or destroyed; verify a test that a scripted session's inventory deltas equal the recorded totals, by source
- [x] 1.3 Save both; verify the save/reload test keeps them identical

## 2. Client

- [x] 2.1 The graph renderer as a pure function; verify unit tests for scaling, short-second marking and width
- [x] 2.2 The graph in the pole info box with a range keybind; verify a snapshot of a short network
- [x] 2.3 The Production tab; verify a snapshot of the reference tree farm after 10 minutes

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that the graphs read well
