# Proposal

## Why

The Crafting tab lists every recipe. Early on most of them can't be made, so the player scrolls past red rows to find the few they can craft right now. The search filter (`add-crafting-search`) narrows by name, but not by what's in the inventory.

## What Changes

- **A craftable-only toggle on the Crafting tab:** a key (`crafting.TOGGLE_CRAFTABLE_ONLY`, default `c`) hides every recipe whose ingredients aren't all in the inventory. Pressing it again shows everything.
- **It's visible:** the list title reads "Recipes (craftable only)" while it's on, and the hint line names the key. Clicking the hint toggles it too.
- **It combines with search:** both filters apply, so "plank" with craftable-only shows only plank recipes the player can make now.
- **The list updates live:** crafting something, picking items up or dropping them changes what's shown. The selection stays on the same recipe when it's still listed, else moves to the nearest one.
- **It's remembered:** the setting is saved with the world's client state (`ClientSave`), off in a new world.
- An empty list says "Nothing craftable yet" with the key to show all recipes.

## Capabilities

### Modified Capabilities
- `crafting`: the craftable-only filter on the recipe list.

## Impact

- **Client:** the toggle state, the filter in `filtered_recipes`, the title, hint and empty-list text, a click target for the hint, and the `ClientSave` field.
- **Config:** the new keybind in `default_config.rs`.
- **Tests:** a snapshot of the filtered list from the starting kit, a test that crafting updates it, search plus craftable-only together, and a save-reload test.
