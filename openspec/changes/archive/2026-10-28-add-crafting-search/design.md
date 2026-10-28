# Design

## Context

- **Items tab:** `ItemDictionaryState { selected, search, searching }`. `handle_item_dictionary_input` types into the search while `searching`, Enter (`dictionary.END_SEARCH`) stops typing and keeps the filter, Escape clears, and `filtered_items(app)` filters.
- **Crafting tab:** `CraftingState.selected` indexes `RecipeHandler::get_recipes()` directly. Up and Down wrap, and Enter crafts `selected`.

## Goals / Non-Goals

**Goals:** the same search, keys and feel on both tabs; one implementation.

**Non-Goals:**
- Filtering to recipes you can craft now.
- Sorting by craftability.
- Searching other tabs (a later change can reuse the helper).

## Decisions

### Shared search
A `SearchBox { text: String, typing: bool }` with:
- `handle(&mut self, keys, key) -> SearchKey`, which returns `Consumed`, `Cleared` or `NotMine`;
- `matches(&self, haystacks: &[&str]) -> bool`.

Both tabs hold one and call it first, before their own keys. It lives in `app/input/search.rs`.

### Crafting filter
- `filtered_recipes(app) -> Vec<usize>` returns indices into `get_recipes()` whose recipe name, result name, result description (from its item sprite) or any ingredient name contains the text.
- `CraftingState.selected` becomes a position in that list. Crafting and the tree map it back to the recipe index.
- After the filter changes, the selection resets to the first match.
- With no matches, the list shows "No recipes match" and Enter does nothing.

### Keybinds
`search.SEARCH` (`s`), `search.DELETE_CHAR` (Backspace) and `search.END_SEARCH` (Enter) replace `dictionary.*`. The crafting tab doesn't use `s` for anything today. Scripts that use `key dictionary.*` are updated.

## Risks / Trade-offs

- Renaming the keybind category leaves an unused `dictionary` block in existing config files. That's harmless, and simpler than keeping two names.
