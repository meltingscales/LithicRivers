# Design

## Context

- `ui::panels::crafting::filtered_recipes` returns the recipe indices the search keeps; drawing, keys and clicks all use it, so one filter there reaches everything.
- `RecipeHandler::can_craft(index, inventory)` already decides the green and red rows.

## Goals / Non-Goals

**Goals:** one key, visible state, works with search and the mouse, survives save and reload.

**Non-Goals:** sorting craftable recipes first; counting how many of a recipe can be made; filtering by what's craftable from raw materials (the crafting tree's job).

## Decisions

- **Filter in one place:** `filtered_recipes` adds `!craftable_only || can_craft(i, inventory)` next to the search test.
- **Key:** `c` on the Crafting tab only; it doesn't clash, since `combat.CLEAR_MOVE_QUEUE` is only read in combat and search typing captures keys first.
- **Selection:** remember the selected recipe's index into the full list, and after any change pick the same recipe or the nearest one still shown.
- **Saved** in `ClientSave` with `#[serde(default)]`, so older saves load with it off.

## Risks / Trade-offs

- [Players forget it's on and think recipes are missing] → The title says so, and the empty list names the key to show everything.
