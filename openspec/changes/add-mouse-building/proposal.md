# Proposal

## Why

LithicRivers is keyboard-first, and that stays. But building a factory tile by tile with direction keys is slow, and players coming from Factorio or Minecraft expect to point and click. The terminal already reports mouse events (crossterm mouse capture is on); today only the boot screen uses the scroll wheel. Mouse building, following Factorio's conventions, makes building fast without taking anything away from keyboard play.

## What Changes

- **The tile under the mouse** is highlighted on the World tab. In Place mode it shows a ghost of the selected hotbar block with its facing.
- **Left click** on a world tile:
  - places the selected hotbar block (in Place mode, or in any mode when the selected slot holds a placeable item);
  - otherwise interacts with the block there (opens a chest, a furnace and so on);
  - otherwise looks at the tile.
- **Right click** breaks or mines the tile (as Break mode does).
- **Scroll wheel** cycles the selected hotbar slot; **Shift + scroll** rotates the next placement.
- **Middle click** picks the block under the mouse, as in Minecraft: the hotbar slot holding that block's item is selected, or the item is put in the current slot if the player has it. Clicking a belt facing west also sets the placement facing to west, so copying a line is quick.
- **Reach:** clicks build and break only within the same reach as the keyboard (the 8 tiles around the player). Farther clicks show "Too far" and fill the look panel instead. The nanobot change can extend reach later.
- **Elsewhere:** clicking a tab in the menu bar switches to it, and the scroll wheel scrolls lists such as the Item Dictionary, the cheat console suggestions and the crafting list.
- **Bindings:** mouse buttons are in the config like keys (a `mouse` category: `PLACE`, `BREAK`, `PICK_BLOCK`, `ROTATE`, `NEXT_SLOT`, `PREV_SLOT`), so they can be remapped or turned off.
- **Headless:** new script steps, `click <col> <row> [left|right|middle]` and `scroll <up|down> [col row]`, so mouse play is testable and scriptable.

## Capabilities

### New Capabilities
- `mouse-input`: Mapping screen cells to world tiles, the hover highlight and ghost, click and scroll actions with reach, tab clicks and list scrolling, the mouse bindings, and the headless mouse steps.

### Modified Capabilities
- `headless-client`: Scripts gain `click` and `scroll` steps.

## Impact

- **Client:**
  - Mouse events are routed through one handler, like keys.
  - The world view remembers its screen rectangle and zoom, so a cell maps to a tile at every zoom level.
  - The hover highlight and ghost render from the stored mouse position.
- **Config:** a `mouse` binding category in `default_config.rs`.
- **Headless:** the new steps, the MCP tool mirrors (`click`, `scroll`), and snapshot scripts for hover and click.
- **Core:** none. Clicks turn into the same `PlayerAction`s and build calls the keys use.
