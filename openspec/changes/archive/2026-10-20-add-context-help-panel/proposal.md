# Proposal

## Why

New players on the World page have to open the Help tab, or remember keys from the tutorial, to know what they can press. What's useful changes with the situation: walking, break mode, place mode, look mode, or standing next to a corpse, an item or an NPC. A small panel that always shows the keys that matter right now makes the game easier to pick up without leaving the World page.

## What Changes

- **Context help panel** on the World page, below the inventory list in the right sidebar. It lists the few keys that apply right now:
  - **Walking:** move, mine, interact, look and the build-mode toggle.
  - **Break or place mode:** the direction keys, plus the hotbar in place mode.
  - **Look mode:** cursor movement and how to leave look mode.
  - **Next to something:** the interact key with what it does ("loot corpse", "talk to Broken Android", "pick up Log").
- **Toggle key `h`** (`ui.TOGGLE_CONTEXT_HELP`) shows or hides the panel. When the panel is hidden, the sidebar still shows a one-line "h: help" hint, so the toggle can always be found.
- **The choice is remembered** in the config file (`ui.SHOW_CONTEXT_HELP`, default on), like the volume settings. Headless runs never write it.
- Key names always come from the keybind config, never hard-coded text.
- **No change in combat:** the combat panel already lists its keys.

## Capabilities

### New Capabilities
- `context-help`: The World-page panel that lists the keys relevant to the player's current situation, and its toggle.

### Modified Capabilities
<!-- none -->

## Impact

- Client:
  - new `ui/panels/context_help.rs`;
  - the sidebar layout in `ui/layout.rs` gets a split for the panel;
  - the toggle is handled with the global UI keys;
  - the logic that finds nearby interactables in `app/input/world_action.rs` is reused, so the panel and the interact key always agree.
- Config: `ui.TOGGLE_CONTEXT_HELP` keybind (`h`) and a `ui.SHOW_CONTEXT_HELP` setting.
- Snapshots: World-page snapshots change (the panel appears in the sidebar). Tab snapshots other than World, and combat, are unchanged.
