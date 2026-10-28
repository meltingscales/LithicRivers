# Proposal

## Why

Menus answer to different keys depending on which one is open:
- Most lists and windows (crafting, body repair, chests, dialogue, the filter picker, ...) move only with the numpad (`movement.MOVE_*`).
- Choose Interaction also takes the arrow keys (`ui.MENU_UP` / `MENU_DOWN`), the Quests list takes only the arrow keys (hard-coded `KeyCode::Up` / `Down`), and the global map hard-codes both.
- A window that ignores an arrow lets it fall through to tab switching, so pressing Left in the body repair window switches tabs behind it.

The mouse is uneven too: `mouse-menus` covers most windows, but the body repair window, the Quests list, the Inventory tab's list and tutorial selection don't respond to clicks.

## What Changes

- **One navigation scheme:** every menu, list and window moves its selection with both the numpad (`movement.MOVE_NORTH` / `SOUTH` / `WEST` / `EAST`) and the arrow keys (`ui.MENU_UP` / `DOWN` / `PREV` / `NEXT`), through one shared helper. Enter / Space (`ui.MENU_ACTIVATE`) activates and Escape (`ui.CLOSE_HELP_MENU`) closes, as today.
- **Left and Right:** in a window with sides or panels (chest and corpse windows, the furnace, the body repair window) they move between them, and the window keeps the key, so tabs never switch behind an open window. On a tab's own list (Inventory, Crafting, Items, Quests) Left and Right keep switching tabs.
- **No hard-coded navigation keys:** the Quests list, the global map and the help scroll read keybinds instead of `KeyCode::Up` / `Down`.
- **Mouse in the remaining menus:** the body repair window (recipes and body parts), the Quests list, the Inventory tab's list and tutorial selection record click areas; a click selects a row, and clicking the selected row again activates it, as in the crafting list.

## Capabilities

### New Capabilities
- `menu-navigation`: Which keys move, activate and close every menu, list and window.

### Modified Capabilities
- `mouse-menus`: adds the body repair window, the Quests list, the Inventory tab's list and tutorial selection.

## Impact

- **Client:** menu navigation helpers on `Keybinds`, used by every menu's input handler; click targets and mouse handling for the four windows above.
- **Tests:** a headless test per menu that both key sets move it; tests that Left in a window doesn't switch tabs; click tests for the new windows.
- **Out of scope:** the cheat console's text editing (Left / Right move its text cursor), and world movement and building.
