# Tasks

## 1. Ruins

- [ ] 1.1 Add `ReactorCasing`, `NuclearWaste`, `UraniumDeposit` tiles, their drops and items, and the mining cost multiplier; verify mining tests for each tile, that casing takes 5× rock's mining cost, and the raw-sources test
- [ ] 1.2 Add three ruin layouts as `.lrstructure` files and seeded placement through pending structures; verify a same-seed placement test, a machine-generated vs walked-to equality test for a ruin crossing a chunk edge, and a test that every layout has at least one uranium deposit and fewer uranium than waste

## 2. Thermoelectric generator

- [ ] 2.1 Add the thermoelectric generator (heat source slot, constant supply into the power grid, never consumed); verify tests for empty, waste and uranium output over many steps, and save/reload with a source loaded
- [ ] 2.2 Add the generator window and its recipe; verify a headless test loading and unloading a heat source, and the crafting reachability test

## 3. Client and docs

- [ ] 3.1 Sprites for the new tiles, items and generator; verify a snapshot script with a ruin in view (via cheat teleport) and the generator window
- [ ] 3.2 Update Help, AGENTS.md and `docs/TECH-TREE.md`; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check: finding and harvesting a ruin, and powering something with a thermoelectric generator feel right
