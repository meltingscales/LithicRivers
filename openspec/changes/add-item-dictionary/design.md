# Design

## Context

- Items: `ItemKind` (strum-iterable) with `itemkind_name` and `itemkind_sprite_name`. Sprite `data.json` files hold a description.
- Recipes: `RecipeHandler`; raw sources: `crafting::raw_sources`, `mining_drop`, `FERAL_DOG_LOOT`.
- Cheat console: `app/input/cheat_console.rs` builds `autocomplete_suggestions`; `ui/modals/cheat_console.rs` joins them into one help line.

## Goals / Non-Goals

**Goals:** see every item and its relationships; search; readable suggestions.

**Non-Goals:** giving items from the dictionary (that is the cheat console's job), bookmarks, mod support.

## Decisions

- **Descriptions** come from the item sprite's `data.json` `description`, so art and text live together. A test fails for any item whose sprite has no description.
- **Search** is a case-insensitive substring match over name and description. A keybind in `default_config.rs` focuses the search field, and Escape clears it.
- **Details** reuse the crafting tree code (`add-crafting-tree`) for "made from", and add `crafting::used_in(item)`.
- **Suggestions** render in a bordered box under the input, up to 8 rows, scrolling to keep the selected suggestion visible. Tab and the up/down keys keep their current meaning.

## Risks / Trade-offs

- [A tab bar that is already crowded] → The Menu tab bar already scrolls; the new tab sits after Crafting.
