# Design

## Context

- `RecipeHandler::new()` holds six `Recipe { name, ingredients, result, quantity }`. `TileKind::from_item_kind` decides what's placeable (`PlankBlock`, `Stone`).
- `mining_system` handles only `Tree` and `PlankBlock`; everything else logs "You can't mine the … tile."
- Worldgen places `IronScrap`, `ScrapElectronics` and `PlasteelScrap` tiles; there's no iron item (an `iron_scrap` item sprite exists).
- `Nail` and `String` appear only in the starting inventory.

## Goals / Non-Goals

**Goals:** every placeable item craftable from gatherable materials; enforced by a test; visible to the player as a tree.

**Non-Goals:** smelting or processing machines, recipe unlocks/progression, crafting stations, balancing beyond sensible starting numbers.

## Decisions

### One placeable list, one raw-source table
`placeable_items()` in core returns every item `TileKind::from_item_kind` or a factory kind accepts. `raw_sources()` lists items obtainable without crafting, built from the mining drop table, tree drops and creature loot tables, so it can't drift from what the game really drops.

### Reachability test
Treat recipes as edges result ← ingredients. Iteratively mark an item obtainable when it's a raw source or some recipe for it has all ingredients obtainable. Fail for each placeable item not marked, reporting the first unobtainable ingredient on its cheapest recipe.

### Mining drops
| Tile | Drop |
|---|---|
| Rock | 1 Stone |
| IronScrap | 1–2 IronScrap (seeded by world seed + position) |
| ScrapElectronics | 1 ScrapElectronics |
| PlasteelScrap | unchanged (not minable yet) |

Mining time follows the existing mining cost.

### Starting recipes
| Recipe | Ingredients | Result |
|---|---|---|
| Nails | 1 IronScrap | 8 Nail |
| String | 1 Leather | 4 String |
| Chest | 8 WoodenPlank, 4 Nail | 1 Chest |

Later changes add: wooden conveyor, burner generator, power pole, auto-planter, auto-harvester (numbers in their designs). All numbers are tunable.

### Tree view
The Crafting panel's detail area renders the selected recipe recursively (indentation, `needed / owned`), using the first recipe for each intermediate item; raw materials end the branch.

## Risks / Trade-offs

- [Iron scrap only in some biomes or structures] → The reachability test checks the recipe graph, not map distribution; a follow-up can check worldgen frequency.
- [Multiple recipes for one item make the tree ambiguous] → Show the first recipe; acceptable while each item has one.
