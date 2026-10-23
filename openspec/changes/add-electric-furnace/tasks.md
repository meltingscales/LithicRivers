# Tasks

## 1. Core

- [x] 1.1 Add the electric furnace kind (`Furnace` without fuel, `PowerConsumer` at 90 kW while smelting); verify `TestWorld` tests for powered smelting, idle demand of zero, half-power doubling time, and unpowered stop
- [x] 1.2 Route back and side inputs to the input slot; verify a belt-fed test

## 2. Client, recipe and checks

- [x] 2.1 Glyph and window variant without a fuel slot, showing power satisfaction; verify a snapshot
- [x] 2.2 Add the recipe (includes a crude furnace); verify the crafting reachability test passes
- [ ] 2.3 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check

## 3. Showcase and demo

- [x] 3.1 A Factory showcase example: a chest of logs, an extractor and long belts into a powered electric furnace, and long belts out to a chest; verify `showcase` tests (every kind has an example, examples make progress)
- [x] 3.2 The factory demo watches that line half a game second a beat and opens the furnace; verify the `demo_factory` snapshot
