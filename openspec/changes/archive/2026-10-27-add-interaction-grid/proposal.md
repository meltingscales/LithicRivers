# Proposal

## Why

When several things are next to the robot, Interact opens "Choose Interaction": a plain list where each row ends in a compass letter ("Open chest (W)", "Loot corpse (E)"). The player has to read the letters and turn them into directions in their head. The user's idea, from playtesting: show the choices as a 3x3 grid, laid out like the tiles around the robot, so you can see which thing is where without remembering N/E/S/W.

## What Changes

- **A 3x3 grid of the tiles around the robot**, laid out like the numpad (and the movement keys), with the robot in the middle. Each cell shows its movement key and the glyph of the thing there in its own color. Cells with something to do are bright; empty cells are dim.
- **The direction keys choose a tile:** pressing a movement key (numpad 7/8/9/4/6/1/2/3, or whatever they're bound to) does that tile's interaction right away. The wait key (5) picks the robot's own tile, for things underfoot.
- **More than one choice on a tile:** the first press selects the cell and lists its choices beside the grid; pressing the same key again moves through them, and Enter chooses. Up and Down (new `ui.MENU_UP` / `ui.MENU_DOWN` keybinds, the arrow keys by default) move through the whole list, so every choice stays reachable.
- **The list beside the grid** names each choice with its key (`[4] Open chest`), replacing the compass letters, and highlights the selected cell's choices.
- **Mouse:** clicking a cell is the same as pressing its key; clicking a list row still chooses it.
- Keys are printed with `get_printable_key_for_keybind`, so rebinding stays correct.

## Capabilities

### New Capabilities
- `interaction-grid`: the Choose Interaction grid, choosing by direction, several choices on one tile, and mouse support.

### Modified Capabilities
<!-- None: mouse-menus' row clicks keep working on the list beside the grid. -->

## Impact

- **Client:**
  - `ui/modals/action_selection.rs`: draws the grid and the keyed list.
  - `app/input/multi_action.rs`: direction keys choose a tile, Up/Down move through the list.
  - `InteractionType::position()` gives each choice's tile.
  - A grid cell click target.
- **Scripts:** `scripts/agent/bugs/choose-interaction-directions.txt` is updated, and a new snapshot covers a tile with two choices.
- **Out of scope:** interactions beyond the eight neighbors (none exist), and changing what the single-interaction shortcut does (one choice still runs at once).
