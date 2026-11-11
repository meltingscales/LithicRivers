# Proposal

## Why

The Steam release targets the Steam Deck (`docs/STEAM-MVP.md`, QA: "Steam Deck compatibility"), but the desktop game can only be played with a keyboard and mouse. A Deck player, or anyone on a couch with a gamepad, can't move, mine or open a menu without one. `docs/SPEC-GAPS.md` lists "Controller support and Steam Deck checks" as missing. Phones already show that a pad can drive the whole game by sending the same keys as the keyboard (`touch-controls` spec). A gamepad can do the same on desktop.

## What Changes

- **Gamepad input on desktop builds:** the game window reads gamepads, including the Steam Deck's built-in controls, which Steam presents as a standard gamepad. Each button sends the key bound to an action, the way the touch pad does, so rebinding and every screen keep working.
- **A default layout** (Xbox names; the Deck's buttons match):
  - Left stick or D-pad: move in 8 directions, or move through menus.
  - A: on the World page, Interact; in windows and menus, Enter.
  - B: Escape (close or back).
  - X: Mine.
  - Y: cycle build modes.
  - LB / RB, tapped: the previous and next menu tab.
  - LB, held: the context wheel (below).
  - LT / RT: the previous and next hotbar slot, as the mouse wheel does.
  - Right stick in Break, Place or Rotate mode: the break or place direction (the `q w e a s d z x c` keys).
  - View (Back): Look mode.
  - Menu (Start): the Menu tab, to save and quit.
  - Left stick click: let time pass.
  - Right stick click: torch.
- **Context wheels:** most actions have no button, so holding LB opens a wheel of up to 8 actions for where the player is:
  - **On the World page:** pick up, torch, pass time, the Inventory tab, blueprint mode, context help, look and the Menu tab.
  - **In build modes:** rotate, the next and previous hotbar slot, the blueprint library, a new blueprint, the next build mode and blueprint mode.
  - **On the Inventory tab:** use, drop and auto-pickup.

  A stick or the D-pad highlights a slot, releasing LB does it, and B cancels. The wheel draws as a 3x3 box around a centre, like the Choose Interaction grid. Wheel contents ship in the default config, as the button layout does.
- **Held directions repeat:** a held stick or D-pad direction repeats, with a delay before the first repeat and a steady rate after it. Times come from the client clock, so runs stay deterministic.
- **Button prompts follow the last device used:** after a gamepad press, key hints show the button instead of the key ("A: interact", "B: close"). A key press switches them back. Hints for actions with no button keep showing the key.
- **Options:** a "Gamepad" row turns gamepad input on or off (on by default) and is saved.
- **Headless scripts:** `pad <button>` (a tap), `pad hold <button>`, `pad release <button>` and `pad <stick> <direction>` drive the same mapping, so controller behavior, wheels included, can be tested and snapshotted.

Out of scope:
- Rebinding gamepad buttons in game.
- Rumble.
- Gamepad text entry: on the Deck, Steam's own on-screen keyboard (Steam + X) types into text fields.
- Gamepad on Android.
- A Steam Input configuration file, which can come later if a Deck check shows it is needed.
- The Steam Deck performance check, which stays its own MVP item.

## Capabilities

### New Capabilities
- `gamepad-input`: Reading gamepads on desktop builds, the default button layout and its context rules, context wheels, held-direction repeat, button prompts after gamepad use, and the Options toggle.

### Modified Capabilities
- `headless-client`: Input steps gain `pad`, which presses a gamepad button or holds a stick direction through the same mapping.

## Impact

- **Client:**
  - A new `crates/client/src/window/gamepad.rs`, polled each frame in `about_to_wait`.
  - A pure mapper from pad events to key presses and mouse-wheel steps, shared with headless runs.
  - The printable-key lookup used by every hint learns the last input device.
  - An Options row.
- **Core:** `gamepad` and `gamepad_wheels` sections in `default_config.rs` naming the action each button and wheel slot sends (the shipped layout), and a `GAMEPAD_ENABLED` setting.
- **Client UI:** a wheel overlay drawn over the game while a wheel is open.
- **Dependencies:** `gilrs`, a cross-platform gamepad library, for Linux (including SteamOS), Windows and macOS. Desktop only; Android keeps the touch pad.
- **Docs:**
  - `AGENTS.md` (the `pad` step)
  - `docs/STEAM-MVP.md` (the controller item)
  - `docs/SPEC-GAPS.md` (remove the controller row's "Missing")
