# Tasks

## 1. Core

- [x] 1.1 `scrap_recipe(item)`: the one recipe that makes an item, or none for raw items and items with several recipes; verify `cargo test` that logs and wooden shavings are refused and a pipe gun returns its recipe
- [x] 1.2 The scrap workbench item, recipe and `FactoryKind::ScrapWorkbench` with its state, power draw (30 kW), batches of the recipe's quantity, 4 factory seconds per batch scaled by satisfaction, seeded 50% rolls per ingredient unit, an output buffer that waits when full, input from back and sides, output to the front, and `factory_stats::record`; register its component for saves; verify `cargo test` for recovery near half over many batches and never above cost, full batches only, refused items, pausing without power, a belt-fed line into a chest, determinism, the reachability test, and `save_reload_anywhere_changes_nothing`

## 2. Client

- [x] 2.1 The machine window (input, output stacks, progress and what the input scraps into), Interact to open it, sprites for the item and block, and Look text; verify a `just client-blind` snapshot of a powered workbench scrapping a pipe gun, the sprite gallery and font tests

## 3. Checks

- [ ] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check: scrapping feels worth doing, and the 50% rate feels fair

## Workflow follow-up

- When merged with only 3.2 left, add a playtest queue entry for it.
