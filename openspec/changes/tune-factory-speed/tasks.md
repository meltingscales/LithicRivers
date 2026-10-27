# Tasks

## 1. Core

- [x] 1.1 `FACTORY_SECOND_TICKS`, `STEPS_PER_FACTORY_SECOND` and a derived 60-tick `FACTORY_STEP_TICKS`; test helpers count factory seconds; verify `cargo test -p lithicrivers-core`
- [x] 1.2 The player rides a belt one tile per factory step (`BELT_RIDE_STEPS_PER_TILE`); verify `a_belt_carries_the_player_a_tile_every_step`

## 2. Scripts and checks

- [x] 2.1 Re-time scripts that ticked less than a step or waited on saplings (`factory-furnace-window`, `factory-electric-furnace-window`, `look-at-sapling`, the factory demo) and review the showcase snapshot diffs; verify `cargo test --workspace`
- [x] 2.2 Verify `just code-clippy` passes
- [ ] 2.3 User check that factories feel well paced in the live client
