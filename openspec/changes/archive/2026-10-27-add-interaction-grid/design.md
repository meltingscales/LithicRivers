# Design

## Context

- `nearby_interactions` (`app/input/world_action.rs`) collects `InteractionType`s within one tile, including the robot's own tile. Each has a position, and `display_name_with_context` appends a compass label.
- The modal (`ui/modals/action_selection.rs`) is a `List`; `multi_action.rs` maps `MOVE_NORTH` / `MOVE_SOUTH` to up and down.
- Movement keys default to the numpad (`NUMPAD_7` … `NUMPAD_3`), and `WAIT` is `NUMPAD_5`.

## Goals / Non-Goals

**Goals:** see where each choice is at a glance; choose with the same key you'd use to step that way; keep every choice reachable by keyboard and mouse.

**Non-Goals:** a general radial menu; showing tiles beyond one step.

## Decisions

### Layout
A grid of 3x3 cells, each 5 columns by 2 rows, with the keyed list to its right:

```text
┌─────────────────── Choose Interaction ───────────────────┐
│  ┌─────┬─────┬─────┐                                     │
│  │  7  │  8  │  9  │   [4] Open chest                    │
│  │     │     │  >  │   [6] Loot corpse (2 items)         │
│  ├─────┼─────┼─────┤   [9] Put Log on belt               │
│  │  4  │  5  │  6  │                                     │
│  │  C  │  @  │  x  │                                     │
│  ├─────┼─────┼─────┤                                     │
│  │  1  │  2  │  3  │                                     │
│  │     │     │     │                                     │
│  └─────┴─────┴─────┘                                     │
│  7-9 / 4-6 / 1-3: choose by direction   Enter: choose    │
│  Up/Down: list   Esc: cancel                             │
└──────────────────────────────────────────────────────────┘
```

- The key in each cell is the printable key for that direction's keybind (so `8` with the defaults).
- The glyph is the 1x sprite of the entity or factory block the first choice there acts on, in its sprite color. A cell with no choice shows its key dimmed and nothing else.
- The selected cell's key and glyph are drawn black on yellow; the selected row is bold yellow and the other rows on that tile are yellow. The list is in the existing order (nearest kinds first), with each row's key in brackets instead of the compass letter.
- The grid is 19 columns wide, so the modal fits at 80 columns. Below 40 columns of modal it falls back to the list alone.

### Keys
- A movement key (or `WAIT` for the middle) whose tile has exactly one choice runs it at once. This is what makes the grid fast: Interact, then the direction.
- If the tile has several choices, the first press selects the first of them; pressing it again cycles; Enter or `MENU_ACTIVATE` chooses.
- A direction with no choice does nothing and logs "Nothing there".
- `ui.MENU_UP` / `ui.MENU_DOWN` (new keybinds, the arrow keys by default) step through the whole list. The selected cell follows the selected row.
- These replace the current use of `MOVE_NORTH` / `MOVE_SOUTH` as list up and down, since those are now directions.

### Mouse
Each cell is a click target (`ClickTarget::InteractionCell(dx, dy)`) that acts like its key; list rows stay `InteractionChoice`.

## Risks / Trade-offs

- **Muscle memory:** players who pressed 8 and 2 to move through the list now pick tiles instead. The arrow keys still move through the list, and the hint line says so.
- **Two things on one tile** (a corpse and an item) take two presses; the list shows both under the same key.
