# Design

## Context

- `main.rs` enables crossterm mouse capture; `Event::Mouse` only handles scrolling on the boot screen.
- **The world view** is rendered at 1x, 2x or 3x zoom around `ui.view_x/y/z`. `bottom_menu_rect` is stored but unused.
- **Building:** `build.PLACE_<DIRECTION>` and `build.BREAK_<DIRECTION>` act on the 8 neighbours, and the hotbar picks the block.
- **Keybinds:** `keybinds.matches(category, action, &key)`; the rule is never to hardcode inputs.

## Goals / Non-Goals

**Goals:** fast point-and-click building with Factorio-like buttons; mouse optional; testable headlessly.

**Non-Goals:**
- drag-to-place lines (a follow-up)
- drag-select for blueprints (`add-blueprints` can add it)
- mouse-driven combat

## Decisions

### One input path
- Mouse events become a `MouseInput { kind, col, row, shift }` handled by `app::input::mouse`.
- Mouse actions call the same functions as keys: placing through the build code, breaking through `PlayerAction::MineAt`, interacting through `execute_interaction_action`, and looking through look mode.
- Nothing in core changes.

### Screen to world
- The world view stores its `Rect` and zoom each frame. `screen_to_tile(col, row) -> Option<Position>` divides by the cell size of the zoom and offsets by the view center.
- Cells outside the view, or in panels, give `None`.

### Bindings
- A `mouse` category in `default_config.rs` maps actions to buttons: `PLACE: ["LEFT"]`, `BREAK: ["RIGHT"]`, `PICK_BLOCK: ["MIDDLE"]`, `ROTATE: ["SHIFT+SCROLL"]`, `NEXT_SLOT: ["SCROLL_DOWN"]`, `PREV_SLOT: ["SCROLL_UP"]`.
- `keybinds.matches_mouse` mirrors `matches`.

### Pick block (middle click)
- Middle click works at any distance, because it only reads.
- On a factory block or a placed block, it finds that block's item:
  - It selects the hotbar slot that already holds the item.
  - Otherwise, if the player carries the item, it assigns the item to the current slot.
  - Otherwise it logs "You have no <item>".
- On a directional block it also sets `placement_facing` to that block's facing. On terrain it does nothing.

### Reach and feedback
- Reach is a Chebyshev distance of 5 from the player (`MOUSE_REACH`), wider than the 8-direction keys so building with the mouse is quick (raised from 1 after playtesting).
- Out of reach, a left click looks at the tile and logs "Too far to build there"; a right click logs "Too far to mine there".

### Hover
- `ui.mouse_cell` is updated on mouse moves (crossterm `Moved` events).
- The World tab draws a highlight on that tile and, in Place mode, the hotbar block's sprite in a dim color. It never changes game state.

### Headless steps
- `click <col> <row> [button]` and `scroll <up|down> [col row]` inject the same `MouseInput`. `move <col> <row>` sets the hover.
- The MCP server gets matching tools. Exit statuses and determinism are unchanged.

## Risks / Trade-offs

- [Terminals without mouse support] → Everything stays reachable by keyboard; mouse bindings can be emptied.
- [Click-through on modals] → While a modal is open, clicks outside it are ignored, as are keys the modal doesn't use.

## Implementation Notes

- **One handler:** `app/input/mouse.rs`. `handle_mouse` ignores events while the demo plays; `dispatch_mouse` is what the demo's scripted mouse steps use. `main.rs` turns crossterm presses, scrolls and moves into `MouseInput`; releases and drags are ignored.
- **Order of a left click on the world:** interact with what's on the tile first (open a chest, talk), then place the selected hotbar block, then look. So clicking a chest opens it even with a chest selected, as in Factorio. This replaces the planned "place when the slot holds a placeable item, otherwise interact".
- **Bindings:** a `mouse` category in `default_config.rs` with button names (`LEFT_CLICK`, `RIGHT_CLICK`, `MIDDLE_CLICK`, `SCROLL_UP`, `SCROLL_DOWN`, `SHIFT+SCROLL_UP`, `SHIFT+SCROLL_DOWN`). The key map skips that category, since it holds button names, not keys.
- **Mapping:** `ui::panels::world::cell_to_tile` is shared by rendering and clicks. The world view stores its `Rect` each frame (`ui.world_view_rect`), and `screen_to_tile` uses it.
- **Reach and look:** building and breaking reach 5 tiles from the player (`MOUSE_REACH`, raised from the keyboard's 8 neighbours after playtesting). Out of reach, a click puts look mode's cursor on the tile and logs "Too far to build there" or "Too far to mine there".
- **Pick block** works at any distance. It selects the hotbar slot holding the block's item, or assigns the item to the current slot if the player carries it, and copies a directional block's facing.
- **Scrolling:**
  - On the World tab with no window open, scrolling cycles hotbar slots, wrapping around.
  - Elsewhere it presses the list's Up or Down key: the cheat console's scroll keys while it's open, otherwise `ui.CREDITS_SCROLL_*`, which the Item Dictionary and Crafting lists use.
- **Tab clicks:** `tab_at` follows the tab bar's layout (a border, then " title " plus a divider for each tab).
- **Hover:** the tile under the mouse is drawn reversed. In Place mode it shows the selected block's sprite at the current zoom and facing.
- **Headless and MCP:** the `click`, `scroll` and `move` steps and the `click`, `scroll` and `move_mouse` MCP tools. They're documented in AGENTS.md, including the player's screen cell at 100x30.
