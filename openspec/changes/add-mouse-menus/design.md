# Design

## Context

- `app::input::mouse` handles world clicks, tab clicks (`tab_at` re-derives the tab bar layout) and list scrolling (by pressing the list's up/down keys).
- Menus and windows are drawn by ratatui widgets in `ui/`; their input lives in `app/input/*` and acts on selection indices (`selected`, `player_item`, `world_selected` and so on).
- Rendering happens every frame in both the interactive and headless loops.

## Goals / Non-Goals

**Goals:** click any row anywhere; behavior identical to the keyboard; no layout logic duplicated between drawing and clicking.

**Non-Goals:** drag and drop between panels; mouse-only menus that the keyboard can't reach; text selection.

## Decisions

### Click targets recorded while drawing
- `ui.click_targets: Vec<(Rect, ClickTarget)>` is cleared at the start of each frame. A widget that draws selectable rows pushes one entry per visible row, with the row's on-screen `Rect` and a `ClickTarget` naming what it is, for example:
  - `TitleChoice(i)`
  - `WorldRow(i)`
  - `CreateField(field)`
  - `InteractionChoice(i)`
  - `TransferRow { panel, row }`
  - `FurnaceSlot(slot)`
  - `FilterRow(i)`
  - `DialogueChoice(i)`
  - `RecipeRow(i)`
  - `DictionaryRow(i)`
  - `CombatMove(i)`
  - `Suggestion(i)`
  - `ModalArea` (the window's outer rect, for click-outside)
- Scrolled lists push only visible rows, with their real indices.
- The tab bar records its tabs too, replacing `tab_at`'s re-derived layout.

### Dispatch
- A click checks the targets from last to first, so the topmost drawn wins, before any world handling.
- A row target sets the matching selection index and then presses that menu's own activate key (`title.SELECT` or `ui.MENU_ACTIVATE`) through `input_handler::dispatch`, so the click runs exactly the code the key runs.
  - *Changed during implementation:* the plan was new `activate_*` functions shared by the key and the click. Dispatching the bound key needs no change to any input handler and can't drift from the keyboard path.
  - Exceptions: a create-form text field only takes focus; an options row other than Back only selects; a recipe row selects on the first click and crafts on a second.
- A click that lands on no target while a window is open, and outside its `ModalArea`, closes it, as Escape does.

### Right click in transfer windows
Moves the whole stack.
- *Changed during implementation:* instead of a new `move_stack`, the click presses activate repeatedly until the stack is gone, a move is refused, or the selection lands on a different item. Every item goes through the window's own rules and messages, so a full furnace slot keeps the rest of the stack.

### Hover
`ui.mouse_cell` is already tracked. After drawing, `ui()` underlines the target a click at the mouse cell would press, using the same `resolve` as clicks, so the hover state can't drift from the click state and rows under an open window never light up.
- Underlines don't show in the text snapshots, so tests check the buffer's cell styles instead of hover snapshots.

### Draw order
The hotbar, message log and menu bar now draw before every window. Before, the message log drew over the bottom of item windows, hiding their key hints, and the menu bar's tabs sat above open windows.

## Risks / Trade-offs

- [A widget forgets to record its rows] → A headless test clicks every recorded target type at least once; buffer-style tests cover hover.
- [Targets from the previous frame after a layout change] → Targets are rebuilt every frame and a click is handled before the next draw changes layout, the same frame order as keys.
