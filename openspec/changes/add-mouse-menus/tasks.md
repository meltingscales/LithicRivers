# Tasks

## 1. Mechanism

- [x] 1.1 `ClickTargets` on the UI state (cleared each frame), dispatch before world clicks, click-outside-closes, and the tab bar recording its tabs (replacing `tab_at`); verify `cargo test` for topmost-wins, click-outside, and tab clicks still working
- [x] 1.2 Clicks press each menu's own activate key, shared with Enter (see design: replaces `activate_*` functions); verify existing keyboard tests still pass

## 2. Menus and windows

- [x] 2.1 Title screen, world list, world actions, delete confirmation and the Create New World form; verify headless click tests for each
- [x] 2.2 Choose Interaction, dialogue choices, filter picker, hotbar block picker, combat moves and cheat suggestions; verify headless click tests for each
- [x] 2.3 Transfer windows (left click moves one, right click moves the whole stack) and the furnace window; verify tests including a machine refusing a right-clicked stack it can't take (a full furnace slot)
- [x] 2.4 Crafting and Item Dictionary lists (select, and craft on a second click), including scrolled lists; verify headless tests
- [x] 2.5 Hover highlighting for every recorded row; verify buffer-style tests for the title screen and a transfer window (underlines don't show in text snapshots)

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check in a real terminal that clicking through menus feels natural
