# Design

## Context

- Keybinds: `movement.MOVE_*` defaults to the numpad, `ui.MENU_UP` / `MENU_DOWN` / `MENU_PREV` / `MENU_NEXT` to the arrow keys. `ui.MENU_PREV` / `MENU_NEXT` switch tabs in `global_ui`, which runs last in `input_handler::dispatch`, so any key a window doesn't consume reaches it.
- Mouse: widgets record `ClickTarget` areas while drawing (`ui/click_targets.rs`), and `app/input/mouse.rs` resolves clicks from them only.

## Goals / Non-Goals

**Goals:** the same keys work in every menu; open windows keep the keys they use; every menu takes clicks.

**Non-Goals:** new keybinds or remapping defaults; changing what any menu does on Enter; the cheat console's text cursor; world movement.

## Decisions

### One helper
`Keybinds::menu_up` / `menu_down` / `menu_left` / `menu_right`: numpad or arrow, through `keybinds.matches`. Every menu's input handler asks these instead of its own pair of `matches` calls, so a remapped key works everywhere at once.

### Who owns Left and Right
- **Windows (modals):** consume Left and Right whether or not they use them, so tabs never change behind them.
- **Tab lists:** only consume Up and Down; Left and Right fall through to tab switching, as today.

### Mouse
Same pattern as the crafting list: the row's `ClickTarget` selects it; a click on the already-selected row dispatches Enter. In the body repair window, the recipe list and the body-part list each get a target variant. The Quests panel wraps its own text (it used ratatui's wrap, which made row positions unknowable), so each quest's title row is recorded exactly; this also keeps the indent the wrap used to trim.

## Risks / Trade-offs

- [Arrow keys in tab lists that used to switch tabs] → Up and Down never switched tabs, and Left and Right still do on tab lists, so nothing a player relies on changes.
- [Many handlers touched] → each gets a headless test with both key sets.
