# Tasks

## 1. World lifecycle (no menus yet)

- [x] 1.1 Add `WorldSettings` to core `Resources` (defaults from config) and make developer-mode checks read it; verify core tests pass and a `TestWorld` test that a world with developer mode off reports it off
- [x] 1.2 Add `Clock::unix_seconds()` (system time for the real clock, fixed epoch + offset for the fake clock); verify a unit test that the fake clock's value only changes when advanced
- [x] 1.3 Add world folders: `WorldMeta` (`meta.json`), folder naming with collision suffixes, listing sorted by last played with unreadable folders flagged, create/save/load/delete (delete refuses paths outside the saves folder), seed parsing (blank, numeric, FNV-1a text hash); verify unit tests for each, including a save → load round trip of position, inventory, tick and settings
- [x] 1.4 Add `App::start_world` / `load_world` / `save_world`; make `App::new_headless` start on a placeholder game and `skip-splash` call `start_world`; give headless runs a temporary saves folder removed on drop; verify `cargo test --workspace` passes with no `.snap` changes and the bug scripts pass

## 2. Title and world screens

- [x] 2.1 Add the `title` keybind category and `ui.SAVE_AND_QUIT_TO_TITLE` to `default_config.rs`; verify a config test that every new keybind parses
- [x] 2.2 Add the title screen (logo, splash text from `config/splashes.txt`, Play Game / Options / Quit) after GameTitle; verify a new snapshot script `scripts/snapshots/title-screen.txt` and an `expect-text "Play Game"`
- [x] 2.3 Add the world list (worlds + Create New World, Play, Delete with confirmation, Back); verify snapshot scripts for the empty list and for a list with one world created earlier in the same script
- [x] 2.4 Add Create New World (name, seed, Allow cheats, Developer mode, Show tutorial, Create / Cancel) leading to the boot intro and then the game; verify a snapshot of the form and a headless test that creating with seed `12345` gives the same `state` as `skip-splash` with seed 12345
- [x] 2.5 Add Options (music and sound volume, applied immediately and saved to config); verify a test that changing a volume updates the config value and the audio manager
- [x] 2.6 Add Rename to the world actions (new name stored in `meta.json`, folder unchanged; back cancels); verify a unit test that only the name changes and a headless test of rename and cancel

## 3. In-game changes

- [x] 3.1 Menu tab: Save writes to the current world folder, Load reloads it, Save and Quit to Title saves and returns to the title screen, and the Menu panel text describes this instead of `save.json`; verify a headless test that save → quit to title → play restores position and inventory
- [x] 3.2 Gate the in-game cheat console on `cheats_allowed` with a "cheats are disabled" message (headless `cheat` steps and MCP not gated); verify headless tests for a world with cheats off and on
- [x] 3.3 Apply Show tutorial from world settings at start/load; verify a headless test that a world created with Show tutorial off has no Tutorial tab

## 4. Docs and checks

- [x] 4.1 Update AGENTS.md and the `play-headless` skill: `skip-splash` starts a world directly, scripts can drive the title menu with `title.*` keybinds, headless saves are isolated; verify the documented example runs with `just client-blind`
- [x] 4.2 Verify `just code-clippy` passes, `cargo test --workspace` passes twice with no pending snapshots, and the PR checks pass on Linux, Windows and macOS
- [ ] 4.3 User check: in the interactive client, the title screen, world list, create-world screen, options, save and quit to title, and replaying a world look and feel right
