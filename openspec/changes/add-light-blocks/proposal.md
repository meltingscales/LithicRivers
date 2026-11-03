# Proposal

## Why

The only light is the one the robot carries: a 2-tile glow, or 8 tiles with a torch. Underground, and on the surface at night, every base and mine shaft falls back into fog as soon as the player walks away, and a factory built in a cave can only be seen a few tiles at a time. The user wants a light block the player can place, so a base, a stairwell or a tunnel stays lit.

## What Changes

- **Scrap lantern:** a new placeable block, crafted from 1 torch and 1 iron scrap. It is placed and broken like other factory blocks (from the hotbar in Place mode; breaking returns the item).
- **It lights the tiles around it:** every tile within 6 tiles (circular, like the player's light) on its own layer, with line of sight, so walls cast shadows exactly as they do for the player's light.
- **Lit tiles count as illuminated:** while the player is on the lantern's layer and near enough for the tiles to be on screen, lit tiles draw in full color and are remembered as seen, the same as tiles the player lights. This works underground and on the surface at night; by day the surface is lit anyway.
- **No fuel, no power:** a lantern burns forever. Light passes over it, like over fences and gates.
- **Look and the Item Dictionary** describe it; its sprite follows `docs/ART-DIRECTION.md` and uses only characters in the bundled font.

Out of scope: powered lamps on the grid (a later change, likely with `add-power-cells`), colored light, light making creatures shy or bolder, light that crosses layers (through stairs or holes), and dynamic light carried by belts or creatures.

## Capabilities

### New Capabilities
- `light-blocks`: Placed light blocks, the scrap lantern, and how their light reaches tiles and joins fog of war.

### Modified Capabilities
None: `day-night-cycle` already says the night surface uses the underground's fog of war and light rules, and those rules now include placed lights through the new spec.

## Impact

- **Core:** `FactoryKind::ScrapLantern` and `ItemKind::ScrapLantern` with its recipe (`factory.rs`, `components.rs`, `recipe_handler.rs`); `fog_of_war.rs` gathers nearby lanterns into `FogLookup` and marks their tiles visited in `update_fog_of_war`.
- **Client:** the sprite, Look text and Item Dictionary entry; a snapshot script.
- **Saves:** lanterns are factory blocks, already saved; the new enum variants need nothing else. `docs/MECHANICS.md` gets a line under Light Sources.
