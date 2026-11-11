# Tasks

## 1. Mapping

- [x] 1.1 Add the `gamepad` layout section to `default_config.rs`, and a config reader that resolves each entry to a keybind action, `context.CONFIRM`, `wheel.UP`/`DOWN`, `build.AIM` or `tab.MENU`, warning on unknown entries; verify with a `cargo test` that the shipped layout resolves completely and every keybind it names exists
- [x] 1.2 Write the pure `Mapper` in `gamepad.rs`: `PadEvent`s plus a `PadContext` and the clock in, key and wheel outputs out. Cover A by context, B always, the right stick by build mode, triggers as wheel steps, `tab.MENU` as `MENU_NEXT` presses, and nothing but B while typing; verify with `cargo test` unit tests for each rule, and that a rebound key changes the output
- [x] 1.3 Stick quantizing (0.5 dead zone, 8 directions), D-pad diagonals from two held buttons, and held-direction repeat (250 ms, then every 120 ms) from the client clock; verify with `cargo test` for the dead zone, every direction, and repeat timing driven by a fake clock

## 2. Context wheels

- [x] 2.1 Add the `gamepad_wheels` config (`WORLD`, `BUILD`, `INVENTORY`, 8 slots each) and `tab.<NAME>` actions. In the mapper: tap vs hold on LB (250 ms), stick and D-pad directions highlighting while a wheel is open, release firing the slot, B cancelling and being swallowed, and LB acting on press where no wheel exists; verify with `cargo test` for each rule and that every shipped slot resolves
- [x] 2.2 Draw the open wheel as a centred 3x3 box with action labels, the highlighted slot reversed and the centre naming it; verify with a `scripts/snapshots/gamepad-wheel.txt` snapshot of the World wheel with a slot highlighted, and one in Place mode

## 3. Window and headless

- [x] 3.1 Add `gilrs` (desktop only), poll it in `about_to_wait`, convert its events to `PadEvent`s, feed the mapper, and dispatch outputs through the existing key and wheel handlers. Skip all of it when Gamepad is off; verify with `cargo test` that a fake event stream reaches the key handler, and that `just build-linux` and the Android build compile
- [x] 3.2 Add the headless `pad <button> [count]`, `pad hold <button>`, `pad release <button>` and `pad <stick> <direction>` steps through the same mapper, failing with status 1 on unknown input; verify with `cargo test` for each form and the error, and a `scripts/snapshots/gamepad-play.txt` snapshot that moves, interacts with a chest, changes tabs and places a block with the right stick
- [x] 3.3 Expose the `pad` step in the MCP server's `run_script` path (no new tool); verify with `cargo test` that an MCP script with `pad` runs

## 4. Prompts and options

- [x] 4.1 Track the last input device on `App` and add `App::key_label`, which gives button names for layout actions in gamepad mode and the key otherwise. Move every `get_printable_key_for_keybind` hint to it; verify with `cargo test` that existing snapshots don't change, and with a snapshot after `pad a` showing "A: interact" and "Hold LB: more actions" in the context help panel
- [x] 4.2 Add the Options "Gamepad" row (on by default, saved as `GAMEPAD_ENABLED`, "not on mobile" on Android); verify with `cargo test` that it toggles, saves and ignores pad steps while off, and with an Options snapshot

## 5. Docs and checks

- [x] 5.1 Document the `pad` steps, the layout and the wheels in `AGENTS.md`. Update `docs/STEAM-MVP.md` and `docs/SPEC-GAPS.md` (the controller row is covered by this change); verify with `just code-clippy` and `cargo test --workspace`
- [ ] 5.2 User check: play with an Xbox-style pad on Linux or Windows; movement, menus, building, combat and every wheel action work without the keyboard, and prompts switch between buttons and keys
- [ ] 5.3 User check on a Steam Deck with Steam's default controller configuration: the built-in controls drive the game, and Steam + X types into the world-name field
