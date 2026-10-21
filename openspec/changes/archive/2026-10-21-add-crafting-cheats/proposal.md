# Proposal

## Why

Testing a recipe means collecting its ingredients first: chopping trees, finding iron scrap, killing dogs for leather. That's slow for players trying things out and for agents scripting crafting checks. `giveitem` helps only if you already know every ingredient and quantity. A cheat that hands over exactly what a recipe needs makes crafting quick to test, and later factory machines quick to try.

## What Changes

- **`givecraftingcomponents <item> [amount]`:**
  - Gives the ingredients needed to craft `amount` of the item (default 1), rounding up to whole crafts.
  - Example: the planks recipe makes 3, so asking for 4 planks gives the ingredients for 2 crafts.
- **`raw` option:** `givecraftingcomponents <item> [amount] raw` gives the raw materials at the bottom of the item's ingredient tree instead, so the whole chain can be crafted from scratch.
- **Feedback:** the message log lists what was given. An item with no recipe, or an unknown item, is reported as an error, and nothing is given.
- **Autocomplete:** item names autocomplete like `giveitem`.
- **Headless:** works through `cheat givecraftingcomponents ...` and the MCP `cheat` tool.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `cheat-console`: adds the `givecraftingcomponents` command.

## Impact

- **Core:**
  - `crafting::components_for(item, amount, raw)`, built on the existing recipe table and `ingredient_tree`, so the cheat and the Crafting panel agree on quantities.
  - Requires `ingredient_tree` to take a craft count.
- **Client:** the command, its autocomplete and its help text in the cheat console.
- **Docs:** AGENTS.md lists the command for headless scripts.
- **Depends on** `add-crafting-tree` (recipe table, ingredient tree).
