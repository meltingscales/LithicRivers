# Tasks

## 1. Core

- [ ] 1.1 Add the electric furnace kind (`Furnace` without fuel, `PowerConsumer` at 90 kW while smelting); verify `TestWorld` tests for powered smelting, idle demand of zero, half-power doubling time, and unpowered stop
- [ ] 1.2 Route back and side inputs to the input slot; verify a belt-fed test

## 2. Client, recipe and checks

- [ ] 2.1 Glyph and window variant without a fuel slot, showing power satisfaction; verify a snapshot
- [ ] 2.2 Add the recipe (includes a crude furnace); verify the crafting reachability test passes
- [ ] 2.3 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check
