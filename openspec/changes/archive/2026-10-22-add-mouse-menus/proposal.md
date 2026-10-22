# Proposal

## Why

Mouse building (`add-mouse-building`) made the world clickable, along with the menu bar tabs and list scrolling. Everywhere else still needs the keyboard: the title screen and world list, the Create New World form, Choose Interaction, the chest, generator and machine windows, the furnace window, the filter picker, NPC dialogue choices, the crafting and Item Dictionary lists, combat moves and the cheat console suggestions. A player who builds with the mouse expects to click through those too.

## What Changes

- **Click any menu row:** one click selects a row and does what Enter would do there.
  - Title screen choices, world list and world actions, delete confirmation.
  - Choose Interaction choices, dialogue choices, the filter picker, the hotbar block picker, combat moves and cheat console suggestions.
  - Crafting recipes and Item Dictionary items (select; clicking the selected recipe again crafts it).
- **Two-panel windows** (chest, corpse, generator, machine, furnace): clicking a row in either panel focuses that panel, selects the row and moves one item, as Enter does. Right-clicking a row moves the whole stack. Clicking a furnace slot selects it.
- **Form fields:** clicking a Create New World row focuses it; clicking a toggle or the World type flips or cycles it; clicking Create or Cancel presses it.
- **Close by clicking outside:** a click outside an open window or menu closes it, like Escape.
- **Hover:** the row under the mouse is highlighted in menus as it is on the map.
- **One mechanism:** widgets record their clickable rows while they draw, so the click handler never re-derives layout, and new menus get mouse support by recording their rows.

## Capabilities

### New Capabilities
- `mouse-menus`: Clicking and hovering rows in menus, lists, forms and windows; right-click moves a whole stack; clicking outside closes a window; the click-target registry.

### Modified Capabilities
<!-- None. `mouse-input` (world clicks, tab clicks, list scrolling) is unchanged; this adds menus. -->

## Impact

- **Client:**
  - A `ClickTargets` list on the UI state, cleared each frame and filled by the widgets that draw rows (`Rect` plus target).
  - `app::input::mouse` dispatches clicks on targets before world clicks.
  - Every menu, list and window listed above records its rows.
- **Headless:** existing `click` and `move` steps drive it; tests check hover underlines in the screen buffer.
- **No core changes.**
