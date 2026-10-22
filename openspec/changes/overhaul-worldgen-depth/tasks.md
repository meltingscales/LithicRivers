# Tasks

## 1. Decisions and survey

- [x] 1.1 User decisions, recorded in design.md: plasteel gets an item; no generator versions, old worlds may show seams
- [ ] 1.2 `worldgen::survey` and a test printing today's table per layer; verify `cargo test` prints it (the baseline in the proposal)

## 2. Generator

- [ ] 2.1 `depth_band`, replacing the old generator; verify z = 0 is identical, an old save still loads with its explored chunks, and the save/reload test
- [ ] 2.2 Sky and shallows terrain, and caves that overlap across layers; verify the survey's pinned bands and overlap
- [ ] 2.3 The feature stage with per-feature generators; verify adding a dummy feature doesn't change deposits
- [ ] 2.4 Deposit tables and clusters, solid deposits, a mining drop for every deposit tile, and the `Plasteel Scrap` item; verify the "nothing unmineable" test and the pinned deposit shares
- [ ] 2.5 Natural stairways and the guaranteed bunker route; verify a test that walks from the bunker to z = 0 on several seeds

## 3. Other changes and docs

- [ ] 3.1 Move nuclear slime spawning onto the feature stage by band (if `add-nuclear-slime` landed first)
- [ ] 3.2 The ore-test debug world shows the bands and deposits; verify its snapshot
- [ ] 3.3 Rewrite `docs/WORLDGEN-ARCHITECTURE.md`

## 4. Checks

- [ ] 4.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass, and review every changed snapshot
- [ ] 4.2 User check: escape the bunker on a few seeds, and digging feels rewarding at each depth
