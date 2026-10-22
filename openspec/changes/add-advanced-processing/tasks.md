# Tasks

## 1. Ore washing

- [ ] 1.1 The washing plant, water input and its seeded loot table; verify `cargo test` for rates over many batches and determinism, and a showcase example

## 2. Petrogen

- [ ] 2.1 The crusher, the distillation column multiblock and the refinery, with slurry, tar, oil and gas; verify tests for each step, an incomplete column doing nothing, and a showcase chain from shale to fuel
- [ ] 2.2 Petrogen fuel and light gas as fuels (the gas generator); verify the fuel table and a generator test

## 3. Uranium

- [ ] 3.1 Crushing, leaching, precipitation and the centrifuge's seeded enrichment; verify tests for ratios, the enrichment rate over a long seeded run, and a showcase chain

## 4. Client and checks

- [ ] 4.1 Machine windows with fluid levels and progress, sprites, and the Item Dictionary showing washing and enrichment chances; verify snapshots and the sprite gallery
- [ ] 4.2 A "every byproduct has a use" crafting test; verify it passes
- [ ] 4.3 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.4 User check that the chains are fun to build and their numbers feel right
