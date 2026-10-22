# Proposal

## Why

Belt lines on one layer can't cross: a line running east blocks every line that wants to run north through it. Factorio solves this with underground belts, a pair of blocks that carry items under whatever lies between them. LithicRivers needs the same so busy floors stay buildable without going up a layer.

## What Changes

- **Underground conveyor:** a pair of blocks on the same layer, an entrance and an exit, facing the same way and at most 10 tiles apart in a straight line.
  - The entrance takes items like a belt, from behind or the sides.
  - Items travel under the tiles between, taking as long as belts would, and come out of the exit, which hands them on in its facing direction like a belt.
  - Whatever is between (belts, machines, walls, rock) is untouched and doesn't matter.
- **Placing:** a placed underground conveyor is an entrance. The next one placed facing the same way, ahead of it in line within range, becomes its exit and links to it. The look panel and a blinking line on the map show which pairs are linked, and an entrance with no exit says so.
- **Breaking** either end unlinks the pair. Items in transit come out at the entrance's tile.
- **Recipe:** 2 wooden conveyors + 2 wooden planks + 1 iron scrap, giving a pair (2 items).
- **Pipes are out of scope** until the game has fluids. This change covers items only.

## Capabilities

### New Capabilities
- `underground-belts`: Underground conveyor pairs, linking, transit under other blocks, and breaking.

### Modified Capabilities
<!-- None. Belts keep their behavior; the belt step gains a link edge used only by underground pairs. -->

## Impact

- **Core:** `FactoryKind::UndergroundEntrance` / `UndergroundExit` (as with vertical conveyors, the role is the kind, so saves and blueprints keep it), an `UndergroundLink { other, in_transit: Vec<(ItemKind, u8)> }` component (registered in `saved_components!`), linking on placement, and a link edge in `belt_order` and `step_belts`.
- **Client:** sprites following `docs/ART-DIRECTION.md` (the entrance and exit differ by glyph), the blinking link line, and the look panel naming the other end.
- **Showcase:** an example where two belt lines cross, one through an underground pair.
- **Blueprints:** a blueprint keeps both ends and re-links them on paste.
