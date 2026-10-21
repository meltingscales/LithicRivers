# Proposal

## Why

There's no in-game way to see every item, what it looks like or what it's for. Developers testing with the cheat console also hit a smaller version of the same problem: `giveitem` suggestions are one comma-joined line that gets cut off, so most item names can't be seen. Minecraft's JEI mod solves both with a searchable item list next to the inventory.

## What Changes

- **Item Dictionary tab** next to Crafting:
  - Lists every item with its 1x sprite, name and a short description.
  - Scrolls, and filters by full-text search over names and descriptions.
  - The selected item shows its 2x and 3x sprites and 12x8 art, the recipe that makes it, the recipes that use it, and where it comes from (mining, creature loot).
- **Cheat console suggestions** become a scrolling list in a floating box under the input, one suggestion per line, with the selected one highlighted, instead of a single line that gets cut off.

## Capabilities

### New Capabilities
- `item-dictionary`: The Item Dictionary tab, its search, and its item details.

### Modified Capabilities
<!-- None. The suggestion-list requirement is part of `item-dictionary`, since both are about finding items. -->

## Impact

- **Core:** an item description for every `ItemKind` (from the sprite's `data.json` description), and a "used in" lookup over recipes, next to `crafting::obtainable`.
- **Client:** a new tab, a search keybind, and the suggestion list in the cheat console modal.
- **Tests:** snapshots of the tab and of the suggestion list, and a test that every item has a description.
