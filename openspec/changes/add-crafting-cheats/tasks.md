# Tasks

## 1. Core

- [ ] 1.1 Give `ingredient_tree` a craft count and add `crafting::components_for`; verify unit tests for direct and raw chests, rounding up (4 planks → 2 logs), and an item with no recipe

## 2. Cheat

- [ ] 2.1 Add `givecraftingcomponents` with argument parsing, logging, errors and autocomplete; verify headless tests for each spec scenario, including crafting a chest after the cheat and status 1 for an unknown item
- [ ] 2.2 Document the command in AGENTS.md and the `play-headless` skill; verify `just code-clippy`, `cargo test --workspace` and PR checks pass
