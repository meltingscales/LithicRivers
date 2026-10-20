# Tasks

## 1. Core

- [ ] 1.1 Add `Generator`, `PowerPole`, `PowerConsumer` and fuel values; generators accept fuel items and refuse others; verify unit tests for fuel acceptance and energy per item
- [ ] 1.2 Add network building (union-find, supply squares, dirty-flag rebuild, stable ids); verify `TestWorld` tests for linked/unlinked poles, merging, splitting on removal, and poles across x = 63/64
- [ ] 1.3 Add per-step distribution with integer joules and per-mille satisfaction; verify tests for idle grid using no fuel, partial load using proportional fuel, remainder kept, shortage giving equal fractions, multiple generators sharing load, and out-of-fuel stop and resume (using a test-only consumer)

## 2. Client

- [ ] 2.1 Generator and pole glyphs; inspect window for supply, demand and fuel; verify a snapshot of the inspect window
- [ ] 2.2 Wire rendering with 1-second blink from the app clock; verify headless snapshots one second apart showing the wire drawn then hidden, identical across runs

## 3. Recipes and checks

- [ ] 3.1 Add generator and pole recipes; verify the crafting reachability test passes
- [ ] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that the grid and blinking wires read well
