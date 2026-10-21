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
- `paste` takes a `BuildCost` policy, `Inventory` for now. `add-nanobot-modules` adds a policy that also needs modules and spreads the placement over time.

## Risks / Trade-offs

- [A large paste breaks the wrong things] → Confirmation with a count, and broken blocks go to the inventory, never lost.
- [The preview clutters the screen] → It blinks, like wires.
