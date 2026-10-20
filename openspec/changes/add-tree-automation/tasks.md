# Tasks

## 1. Core

- [ ] 1.1 Add the `Machine` component and power-aware progress; extract tree chopping into a shared function; verify existing tree-chop tests still pass
- [ ] 1.2 Add the auto-planter (input from belts/player, fixed planting order, radius 2); verify `TestWorld` tests for planting order, belt feeding, no acorns, no free tile
- [ ] 1.3 Add the auto-harvester (fixed chopping order, output buffer, push into faced block, stop when full); verify tests for harvesting onto a belt, into a chest, full output, and identical drops to manual chopping
- [ ] 1.4 Verify power behavior: idle machines demand nothing, half power doubles action time, unpowered machines don't work; and a harvester area spanning four chunks

## 2. Recipes, client and the reference farm

- [ ] 2.1 Add both recipes; verify the crafting reachability test passes
- [ ] 2.2 Machine glyphs and inspect window; verify a snapshot of the inspect window
- [ ] 2.3 Add `scripts/agent/farm.txt` and a `cargo test` that runs it for 30 game minutes and checks logs produced exceed logs burned, deterministically; add a snapshot of the running farm
- [ ] 2.4 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that building and watching the farm is fun
