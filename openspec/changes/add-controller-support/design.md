# Design

## Context

- **Input path:** the game takes crossterm `KeyCode`s and mouse events at cells, whatever the frontend. The game window (`window/mod.rs`) turns winit events into these (`window/input.rs`), and headless runs feed the same handlers from script steps (`headless.rs`, `key` / `rawkey` / `scroll`).
- **Prior art:** the touch pad (`window/touch.rs`) sends `Press::Keybind(category, action)`, which resolves to the first key bound to that action. A gamepad layout can reuse the same idea.
- **Frames:** `about_to_wait` runs `crate::step` once a frame (`FRAME`). Time comes only from `app.clock`.
- **Window settings:** `TOUCH_CONTROLS` lives in `window/settings.rs` and is saved to the config's `ui` section. Gamepad on/off can sit beside it.
- **Hints:** every hint goes through `config_manager.get_printable_key_for_keybind(category, action)` (24 client files).
- **Build modes:** `BuildMode` (Movement, Break, Place, Rotate, Blueprint) is client state.

## Goals / Non-Goals

**Goals:**
- One pure mapping from pad events to game input, shared by the window and headless runs, so it is unit tested and scriptable.
- No game logic knows about gamepads. They only produce the key and wheel input the game already handles.

**Non-Goals:**
- Analog movement or a free cursor. The game is grid-based, so the stick quantizes to 8 directions.
- Pointer emulation for mouse-only features. There are none the keyboard can't reach.
- Changing what any key does.

## Decisions

### `gilrs` for reading pads
- **Choice:** `gilrs` reads gamepads on Linux (evdev, including SteamOS), Windows (XInput / WGI) and macOS, with SDL-style standard button names. It is polled, so `about_to_wait` drains its events each frame.
- **Why not winit:** winit 0.30 has no gamepad events.
- **Why not Steam Input's action API:** it would tie input to the `steam` feature and to Steam running.
- **Steam Deck:** Steam's default Deck configuration presents the built-in controls as an Xbox-style pad, which `gilrs` reads like any other.
- The dependency sits under `cfg(not(target_os = "android"))`.

### A pure mapper: `PadEvent` → `PadOutput`
- **Where:** the mapper is in `crates/client/src/gamepad.rs`, shared by the window and headless runs; only the `gilrs` polling sits in `window/gamepad.rs`.
- **Shape:** `gamepad::Mapper` takes abstract events, `Button(PadButton, pressed)` and `Stick(Stick, Direction or None)`, plus the current client time. It returns outputs: `Key(KeyCode)` or `Wheel(up or down)`.
- **Context:** it reads a small `PadContext` snapshot of the app: on the World page with no window open, the build mode, and whether text is being typed.
  - With no window open on the World page, A becomes Interact; otherwise it is Enter.
  - In Break mode the right stick sends break keys, in Place and Rotate modes place keys, and otherwise nothing.
- **Callers:** the window converts `gilrs` events into `PadEvent`s. The headless `pad` step builds them directly, so both go through identical code.
- **Stick quantizing:** a dead zone of 0.5 on the stick's magnitude, then the angle snaps to the nearest of 8 directions. The D-pad maps straight to its 4 directions, and diagonals come from two D-pad buttons held together.
- **Repeat:** the mapper keeps the held direction and the time of its next repeat. Each frame it is ticked with the clock: the first press sends at once, the first repeat comes after 250 ms, then one every 120 ms. All of it runs from `app.clock`, so headless runs stay deterministic. A headless `pad <stick> <dir>` is press-and-release, with no repeat. `wait` drives repeat for a held step if one is ever added, but no step holds yet.
- **While typing:** pad input is ignored while `App::typing()` holds, except B (Escape), so a search or rename can be left.

### The layout in the default config
- **Where:** a `gamepad` section in `default_config.rs` maps each button or stick to an action string, for example `"SOUTH": "context.CONFIRM"`, `"EAST": "ui.CLOSE_HELP_MENU"`, `"WEST": "action.MINE"`.
- **Special values:**
  - `context.CONFIRM`: the A rule above.
  - `wheel.UP` / `wheel.DOWN`: the triggers.
  - `build.AIM`: the right stick.
  - `tab.MENU`: jump to the Menu tab.
- **Why config:** the shipped layout stays readable and one place to change, and later in-game rebinding can edit it. Unknown entries are ignored with a warning, as keybinds are.
- **Tabs:** `tab.MENU` and `tab.<NAME>` set the tab directly (`PadOutput::Tab`), outside fights and text fields. There is no single key for "go to tab X".

### Context wheels
- **Tap or hold:** the mapper tracks LB's press time.
  - A release within `HOLD_MS` (250 ms) is a tap: LB's own action (`ui.MENU_PREV`).
  - Still held at `HOLD_MS`, as checked in `tick` from the clock, it opens the wheel for the context.
  - Where the context has no wheel, LB acts on press, so menus feel instant.
- **While open:** stick and D-pad directions set the highlighted slot instead of their usual output, without repeat. B cancels, and is swallowed so it doesn't also close a window. Releasing LB fires the highlighted slot's action through the same code as a button, and nothing if no slot is highlighted.
- **Contents:** a `gamepad_wheels` config section with `WORLD`, `BUILD` and `INVENTORY` lists of 8 action strings in N, NE, E, SE, S, SW, W, NW order. Each uses the same vocabulary as buttons, plus `tab.<NAME>` for a menu tab; an empty string is an empty slot.
- **Context:** `PadContext` gains `wheel: Option<WheelKind>`. `WORLD` is the World page, nothing open, movement mode; `BUILD` the same in Break, Place or Rotate; `INVENTORY` the Inventory tab with nothing open.
- **Labels:** a short table names common actions ("pick up", "torch", "rotate"). Anything else falls back to the action's name in lower case with spaces.
- **Display:** the mapper exposes the open wheel (kind, highlighted slot), and `ui/modals/gamepad_wheel.rs` draws it as a centred 3x3 box of 14-character cells. The highlighted slot is reversed, and the centre names it or says "push a direction". It is drawn last, over everything. It is client state only, never saved.
- **Why not a free-angle radial:** the game is a character grid, and 8 directions match the stick quantizing and the existing interaction grid.

### Prompts follow the last device
- **State:** `App` gets `ui.last_input: InputDevice { Keyboard, Gamepad }`. A key or mouse event sets Keyboard, and a pad event sets Gamepad.
- **Lookup:** the client wraps the printable-key lookup in one helper, `app.key_label(category, action)`.
  - In gamepad mode it returns the button whose layout entry sends that action: "A", "B", "X", "Y", "LB", "RB", "LT", "RT", "View", "Menu", "L3", "R3", "D-pad", "L-stick".
  - Otherwise, or for an action with no button, it returns the key as before.
  - Movement actions show "L-stick".
- **Migration:** the 24 call sites move to the helper mechanically. Snapshots don't change, because headless runs start in Keyboard mode.
- **Why Xbox names:** Steam Deck and Xbox share face-button labels and positions; PlayStation labels can come later from the pad's reported type.

### Options row
- A "Gamepad: on/off" row next to Touch controls, saved as `GAMEPAD_ENABLED` in the config's `ui` section.
- On Android the row reads "Gamepad: not on mobile" and does nothing, mirroring the touch row on desktop.

## Risks / Trade-offs

- [A Deck run uses a Steam Input template that maps the pad to keyboard and mouse] → Then gamepad events don't arrive, but the keys do, and the game works either way. The Deck user check covers it.
- [Some pads report the wrong layout on Linux] → `gilrs` ships the SDL controller database. The Options toggle lets a player turn the pad off if it misbehaves.
- [Holding LB to open a wheel delays its tab action until release] → Only where a wheel exists, and a tap is under 250 ms. Elsewhere LB acts on press.
- [Hints in gamepad mode name buttons the player may not have] → Only actions in the layout get button names. Everything else keeps its key.

## Migration Plan

New input only; nothing saved changes except the new option, which defaults on. Rollback is removing the feature; old configs simply lack the `gamepad` section and fall back to the default.

## Open Questions

- Whether to show PlayStation glyphs when a DualSense is connected. This is purely cosmetic and can follow the user check.
