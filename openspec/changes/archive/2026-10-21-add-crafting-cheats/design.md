# Design

## Context

- **Cheat commands** are registered in `app/input/cheat_console.rs` with a name, description, handler and autocomplete. Handlers return `Err` for failures, which the console logs in red and headless runs treat as a script error (`cheat-console` spec, "Failures are reported, never fatal").
- **Recipes** live in `RecipeHandler::new()`. `crafting::ingredient_tree(recipes, recipe, raw)` expands one craft down to raw materials (from `add-crafting-tree`).
- **Item names** are the sprite names (`parse_itemkind_from_sprite_name`), the same as `giveitem`.

## Goals / Non-Goals

**Goals:** one command that sets up any crafting test, with quantities that match the Crafting panel's tree.

**Non-Goals:** auto-crafting; giving machines' inputs (fuel, fluids); multiple recipes for one item (the first recipe is used, like the tree view).

## Decisions

### Quantities
- **Crafts:** `crafts = ceil(amount / recipe.quantity)`.
- **Direct mode:** each ingredient × `crafts`.
- **Raw mode:**
  - `ingredient_tree` gains a craft count, so it expands `crafts` crafts; the existing call becomes `crafts = 1`.
  - The result sums the leaf lines (raw materials, or items that end a branch) by item.
- **Shared code:** `crafting::components_for(recipes, item, amount, raw) -> Result<Vec<(ItemKind, u32)>, NoRecipe>` holds this logic, so it's tested in core and the cheat stays thin.

### Arguments
The arguments after the item are parsed in any order: a positive number is the amount, and `raw` sets raw mode. Anything else is an error that names the argument.

### Errors
An unknown item or an item without a recipe returns `Err`, so headless scripts fail with status 1, consistent with `spawn_enemy`.

## Risks / Trade-offs

- [Ingredients that are themselves raw, like logs in raw mode] → They appear as leaves with their own quantity, as in the tree view.
- [Inventory full] → `Inventory::add` drops what doesn't fit; the log shows what was actually added.
