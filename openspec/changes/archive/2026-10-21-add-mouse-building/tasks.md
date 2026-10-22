# Tasks

## 1. Input plumbing

- [x] 1.1 `MouseInput`, mouse bindings in `default_config.rs`, and the headless `click` / `scroll` / `move` steps (plus MCP tools); verify parser tests and that an unbound button does nothing
- [x] 1.2 Screen-to-tile mapping at every zoom; verify unit tests for 1x, 2x and 3x, and for cells outside the view

## 2. Actions

- [x] 2.1 Left and right clicks on world tiles with reach, look fallback and messages; verify headless tests for placing, interacting, mining and "too far"
- [x] 2.2 Scroll for hotbar slots, the rotate binding, middle-click pick block, tab clicks and list scrolling; verify headless tests for each, including picking a belt's facing and having nothing to pick
- [x] 2.3 The hover highlight and Place-mode ghost; verify a snapshot

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.2 User check in a real terminal that clicking and scrolling feel right
