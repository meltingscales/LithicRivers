# Proposal

## Why

The Crafting tab lists every recipe in one long list, and it grows with every change (furnaces, logistics, stairs, conveyors). Finding a recipe means scrolling. The Items tab already has a search. The user's request: the same search on the Crafting tab.

## What Changes

- **Search on the Crafting tab:**
  - The search key starts typing and Enter keeps the filter.
  - Escape clears it.
  - Backspace erases a character.
  - These are the same keys and behavior as the Items tab.
- **What it matches:** case-insensitive, against each recipe's name, its result item's name and description, and its ingredients' names. So "plank" finds the Wooden Plank recipe and every recipe that uses planks.
- **The list and details follow the filter:**
  - The title shows "Showing N of M".
  - The selection stays on a shown recipe.
  - Crafting with Enter and the ingredient tree act on the selected recipe.
  - Clicks map to the recipe shown in that row.
- **One search for both tabs:** the search state, the key handling and the matching move into a shared helper. The keybinds move from `dictionary.*` to a shared `search.*` category (`SEARCH`, `DELETE_CHAR`, `END_SEARCH`, same default keys), so rebinding them applies to both tabs.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `crafting`: the Crafting tab filters recipes by a search.
- `item-dictionary`: its search uses the shared `search.*` keybinds.

## Impact

- **Client:**
  - `app/input/crafting.rs` and `ui/panels/crafting.rs` work on a filtered list of recipe indices instead of indexing recipes directly.
  - A shared search helper is used by the Item Dictionary and Crafting inputs.
  - Mouse rows map through the filter.
- **Config:** `search.*` keybinds in `default_config.rs`. Old `dictionary.*` entries in existing config files are ignored, and the new ones are filled in from the defaults.
- **Scripts:** `scripts/snapshots/item-dictionary.txt` and the mining-crafting demo switch to `key search.*`; a new snapshot covers a Crafting search.
