# Tasks

## 1. Core

- [ ] 1.1 Cables and networks (flood fill on change, blocks touching two networks); verify `cargo test` for joining, splitting and rebuild on break
- [ ] 1.2 Signals: sensors (the animal counter, storage reader, power reader) and the per-step evaluation order; verify tests for sums and determinism
- [ ] 1.3 The expression parser and evaluator (integer math, division by zero gives 0); verify tests for the grammar, errors and edge cases
- [ ] 1.4 Enable conditions on machines, extractors, routers and generators, with the "waiting" reason; verify tests for each kind of block
- [ ] 1.5 Comparators, calculators and memory cells reading the previous step's values; verify tests that placement order doesn't change results

## 2. Client

- [ ] 2.1 The logic editor (signal list, expression input with autocomplete, errors), cable sprites and look-mode signals; verify snapshots and a headless test setting a condition
- [ ] 2.2 Showcase examples (a farm that stops at 12 sheep, a generator that burns only on high demand); verify the examples-run test

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that logic is easy to set up and debug
