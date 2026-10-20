# Tasks

## 1. Shared logic and config

- [x] 1.1 Extract `nearby_interactions` from the interact handler and use it there; verify the existing bug scripts (including `loot-empty-corpse.txt`) and `cargo test --workspace` pass with no snapshot changes
- [x] 1.2 Add the `ui.TOGGLE_CONTEXT_HELP` keybind (`h`) and the `ui.SHOW_CONTEXT_HELP` setting (default on); verify the default-keybind parse test covers it

## 2. Panel

- [x] 2.1 Add `ui/panels/context_help.rs` with content for movement, break, place and look modes and nearby interactions; verify unit tests for the lines chosen in each situation, including a remapped interact key
- [x] 2.2 Fit the panel into the World sidebar (below the inventory or the look panel; hidden in combat; one-line hint when off); verify new snapshot scripts `scripts/snapshots/context-help-walking.txt`, `context-help-place-mode.txt` and `context-help-hidden.txt`, review and accept the changed World snapshots, and confirm non-World and combat snapshots are unchanged
- [x] 2.3 Toggle with `h`, saved to config except in headless runs; verify a headless test that toggling flips the setting and leaves the config file untouched

## 3. Docs and checks

- [x] 3.1 Mention the panel and the toggle key in the Help tab text and in docs where controls are listed; verify `just code-clippy` and `cargo test --workspace` (twice, no pending snapshots) pass
- [x] 3.2 User check: on the World page, the panel's hints are correct and readable while walking, in break and place mode, in look mode and next to a corpse or NPC, and `h` hides and shows it
