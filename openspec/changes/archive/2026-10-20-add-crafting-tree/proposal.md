# Proposal

## Why

Factory blocks are useless if players can't make them. Today there are six recipes, only `PlankBlock` and `Stone` can be placed, and several ingredients can't be obtained at all: mining only works on trees and plank blocks, there's no iron item, and `Nail` and `String` only exist in the starting inventory. Every placeable block needs a crafting tree that bottoms out in things the player can actually gather.

## What Changes

- **Raw materials from the world**: mining rock gives stone, iron scrap tiles give a new `IronScrap` item, scrap electronics tiles give scrap electronics. These drops, plus creature loot and tree drops, form one table of raw sources in core.
- **Missing intermediate recipes**: nails from iron scrap, string from leather.
- **A recipe for every placeable block**: chests now; every later factory change (belts, generators, poles, planters, harvesters) adds its own recipes in the same table.
- **Crafting-tree check**: a test walks the recipe graph and fails if any placeable item can't be crafted from raw sources, naming the item and the missing link. Later changes can't add an uncraftable block.
- **Crafting panel**: selecting a recipe shows its full ingredient tree down to raw materials, with how many of each you have.

## Capabilities

### New Capabilities
- `crafting`: Raw material sources, recipes, the guarantee that every placeable item is craftable from raw sources, and the ingredient-tree view.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `ItemKind::IronScrap`, mining drops for rock / iron scrap / scrap electronics, `raw_sources()` table, new recipes, reachability test.
- Client: ingredient tree in the Crafting panel; new snapshot.
- Depends on `add-factory-foundation` for the chest item; each later factory change adds recipes and must keep the reachability test passing.
