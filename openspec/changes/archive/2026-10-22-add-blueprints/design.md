# Design

## Context

- Build modes: Movement, Break, Place and Rotate (`build.TOGGLE_BREAK_PLACE_MODE`), with `placement_facing` and the hotbar picker.
- `factory::place` / `break_block` own placing and breaking. Plain placed blocks go through `TileKind::from_item_kind`.
- Blinking already exists for wires (every second of `app.clock`).

## Goals / Non-Goals

**Goals:** copy, rotate and paste player builds; never destroy anything without asking; deterministic.

**Non-Goals:** sharing blueprints between worlds (later), copying block contents, construction robots that fly to the site (the nanobot change decides how building happens).

## Decisions

### What a blueprint holds
- An entry per placed block in the area: its offset from the top-left corner, what it is (a `FactoryKind` or a placeable `TileKind`), its facing, and its filter if it has one.
- Contents, items on belts, saplings and terrain are left out.

### Rotating
- Rotating turns the offsets 90 degrees clockwise about the blueprint's own area and turns every facing with `Facing::rotate_cw`.

### Pasting is all-or-nothing
1. The cost is the sum of each block's item. If the inventory is short, nothing is placed and the message lists what's missing.
2. Every target tile must be free terrain or a player-built block. Rock, trees and other solid terrain block the paste.
3. If any targets hold player-built blocks, the confirmation lists how many. Confirming breaks them into the inventory first (their contents spill as usual), then places everything.

### Preview
- Blueprint tiles draw over the world in the blueprint's sprites, shown on even seconds of `app.clock`.
- Tiles that would be replaced draw in yellow, and tiles that block the paste in red.

### Nanobot hook
- *Changed during implementation:* no `BuildCost` policy yet. With one policy it would be an unused extension point, so `paste` pays from the inventory directly. `add-nanobot-modules` adds the policy when there is a second one.

### Decided during implementation
- **Entering the mode:** `build.BLUEPRINT_MODE` (`b`) enters and leaves Blueprint mode. It is outside the Movement, Break, Place and Rotate cycle, so existing scripts and habits that cycle the modes are unchanged; the mode key from Blueprint mode goes to Movement.
- **The cursor:** an offset from the player, moved with the place-direction keys or by the mouse over the world, so it walks with the player. It is the paste's top-left corner. `PLACE_CENTER` (or a left click) marks a corner or pastes. `NEW_BLUEPRINT` (`n`) starts a copy, and Blueprint mode starts copying while the library is empty.
- **Confirming a replace:** the first paste says how many blocks it would replace; pasting again at the same spot and rotation confirms. Moving or turning cancels. This keeps the flow on one key instead of a dialog.
- **Contents of replaced blocks** go to the inventory with the block, and drop at the player's feet if it is full. Left on the tile, they would block the paste.
- **Built tiles:** only plank blocks count as player-built plain blocks. Placed stone can't be told apart from generated rock, so rock is terrain and blocks a paste.
- **The Blueprints panel** is a window (`build.BLUEPRINT_LIBRARY`, `B`), not a menu tab, so the tab bar and every tab-index script stay as they are. It lists each blueprint's size and cost; `blueprints.RENAME` and `blueprints.DELETE` act on the selection.
- **Text snapshots don't show colors**, so the help panel also says how many tiles block the paste or would be replaced.

## Risks / Trade-offs

- [A large paste breaks the wrong things] → Confirmation with a count, and broken blocks go to the inventory, never lost.
- [The preview clutters the screen] → It blinks, like wires.
