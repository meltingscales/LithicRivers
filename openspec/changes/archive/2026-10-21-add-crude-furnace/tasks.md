# Tasks

## 1. Core

- [x] 1.1 Add `ItemKind::Biochar` (fuel value 4 MJ, accepted by burner generators) and the shared `SmeltingRecipe` table; verify unit tests for each table entry and that non-table items are refused
- [x] 1.2 Add `Furnace` and the crude furnace kind with fuel-only-while-smelting accounting; verify `TestWorld` tests for smelting a log, batch input, idle furnace keeping fuel, pausing and resuming on fuel, and partial fuel energy kept
- [x] 1.3 Add side routing (`insert(item, from)`): back → input, sides → fuel, front push to the faced block; verify tests for a belt-fed furnace with fuel from the side, wrong-side refusal, and a furnace on a chunk edge

## 2. Client, recipe and checks

- [x] 2.1 Furnace glyph (lit while smelting) and the three-slot window; verify a snapshot of the window mid-smelt
- [x] 2.2 Add the crude furnace recipe; verify the crafting reachability test passes
- [x] 2.3 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass; user check that the furnace and its window feel right
