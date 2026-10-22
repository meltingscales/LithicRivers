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
- A row target sets the matching selection index and then runs the same function the Enter key runs: the input handlers get small `activate_*` functions that both the key and the click call.
- A click that lands on no target while a window is open, and outside its `ModalArea`, closes it, as Escape does.

### Right click in transfer windows
Moves the whole stack: `factory::move_all_items` for one stack (a new `move_stack` next to it), through the block's own `accepts` rules so filtered machines still refuse what they don't take.

### Hover
`ui.mouse_cell` is already tracked. Widgets that record targets also style the row under the mouse (underlined or highlighted), so the hover state can't drift from the click state.

## Risks / Trade-offs

- [A widget forgets to record its rows] → A headless test clicks every recorded target type at least once; snapshot scripts cover hover.
- [Targets from the previous frame after a layout change] → Targets are rebuilt every frame and a click is handled before the next draw changes layout, the same frame order as keys.
