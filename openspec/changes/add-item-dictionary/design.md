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

## Implementation Notes

- **Descriptions live in the client.** They come from each item sprite's `data.json`, which core can't read, so `description`, `matching_items` and the "every item has a description" test are in `ui/panels/item_dictionary.rs`. Core gets `crafting::made_by`, `used_in` and `raw_sources_of` (with `RawSource::Mined(TileKind)` / `FeralDog`).
- **Tab order is one list.** `MenuTab::ORDER` drives `next`, `prev`, `as_index` and the menu bar titles, so a new tab is one line. The tab is titled "Items" in the bar and "Item Dictionary" on the panel. Tabs after Crafting moved one place, so the scripts and tests that reach Menu, Help, Credits or Quit press Next one more time, and every snapshot's tab bar changed.
- **Keybinds:** a new `dictionary` category with `SEARCH` (`s`), `DELETE_CHAR` (Backspace) and `END_SEARCH` (Enter). Escape clears the search. Up/Down and the movement keys move the selection.
- **Details:** name, description, raw sources ("Mined from tree"), "Made by" and "Used in" recipe lines on the left; the 12x8 art and the 3x and 2x sprites in a 14-column strip on the right.
- **Cheat console:**
  - Suggestions are a bordered list sized to its contents (up to 8 rows) that scrolls to keep the pick visible; the pick is marked `>`.
  - The scroll keys move the pick while suggestions are shown. Enter on a pick fills it into the input, and a second Enter runs it. Tab works as before.
  - The console is taller (60%) while suggestions are listed.
