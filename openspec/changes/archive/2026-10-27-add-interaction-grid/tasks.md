# Tasks

## 1. Input

- [x] 1.1 `InteractionType::position()` and grouping choices by direction; movement keys and `WAIT` choose a tile (run at once, cycle when several), `ui.MENU_UP` / `ui.MENU_DOWN` (new keybinds, the arrow keys by default) move through the list, an empty direction logs "Nothing there"; verify headless tests for one press, two choices on one tile, arrow keys and an empty direction

## 2. Rendering and mouse

- [x] 2.1 Draw the 3x3 grid (printable keys, glyphs in sprite colors, dim empty cells, yellow selected cell) and the keyed list, with the list-only fallback for narrow modals; verify snapshots: update `scripts/agent/bugs/choose-interaction-directions.txt` and add `scripts/snapshots/interaction-grid-two-on-a-tile.txt`
- [x] 2.2 `ClickTarget::InteractionCell` for each cell; verify a headless click test

## 3. Checks

- [x] 3.1 Update the help text and `AGENTS.md` where they mention the Choose Interaction keys
- [x] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.3 User check that choosing by direction feels natural in the live client
