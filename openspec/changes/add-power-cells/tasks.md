# Tasks

## 1. Core

- [ ] 1.1 The battery block with stored energy, charging from surplus and discharging into deficits; verify `cargo test` for both, rates, and save/reload
- [ ] 1.2 `ItemStack::charge_j` (serde default) and power cells in three tiers, one per stack; verify old saves load and the save-reload identity test
- [ ] 1.3 Charging cells in a battery's slot, and fusion-core trickle charging by core state; verify tests for each rate
- [ ] 1.4 `use_power` for gear; verify a test drawing from the fullest cell and refusing when empty

## 2. Client

- [ ] 2.1 The battery window, cell charge in the inventory and Item Dictionary, and the pole info box's storage line; verify snapshots
- [ ] 2.2 A showcase battery bank that keeps a machine running past a fuel gap; verify the examples-run test

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that batteries and cells are easy to understand
