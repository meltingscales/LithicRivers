# Tasks

## 1. Materials

- [x] 1.1 Add `ItemKind::IronScrap` (name, sprite) and mining drops for rock, iron scrap and scrap electronics tiles (iron scrap quantity seeded by seed + position); verify `TestWorld` mining tests for each tile and determinism of the iron scrap quantity
- [x] 1.2 Add `raw_sources()` built from the mining drop table, tree drops and creature loot; verify a unit test listing the expected raw items

## 2. Recipes

- [x] 2.1 Add nails, string and chest recipes and `placeable_items()`; verify crafting tests for each recipe
- [x] 2.2 Add the reachability test over all placeable items; verify it passes and that it fails, naming the item, when a test-only placeable item with no recipe is added

## 3. Client

- [x] 3.1 Show the ingredient tree for the selected recipe in the Crafting panel; verify a snapshot script selecting the chest recipe
- [x] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that the tree reads clearly in the interactive client
