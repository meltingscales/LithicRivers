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
- Reach is a Chebyshev distance of 1 from the player, matching the 8-direction keys.
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
