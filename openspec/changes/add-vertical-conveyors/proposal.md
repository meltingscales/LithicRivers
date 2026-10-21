# Proposal

## Why

LithicRivers is layered: every z-level is a full world, and the best materials are deep underground. Belts can only move items within one layer, so a mine below can't feed a factory above, and factories can't be stacked. Vertical conveyors move items up or down a layer, opening real 3D factories.

## What Changes

- **Vertical conveyor:** a belt-like block that carries items to the layer above (up) or below (down).
  - It takes items like a belt (from any side, or dropped on it).
  - It hands them to the block directly above or below, which can be a belt, a chest, a machine or another vertical conveyor, so stacks of them climb several layers.
- **Same recipe as a wooden conveyor**, plus two conversion recipes: 1 wooden conveyor to 1 vertical conveyor, and back.
- **Up or down:** a placed vertical conveyor goes up by default, and Rotate mode toggles it between up and down.
- **The layer above or below must be open:** the tile it feeds needs to be passable or a factory block. A conveyor facing solid rock just holds its item.
- **Belts flowing between layers:** a belt line ending in an up conveyor continues from whatever is directly above.

## Capabilities

### New Capabilities
- `vertical-conveyors`: The vertical conveyor block, up/down direction, handoff between layers, and the conversion recipes.

### Modified Capabilities
<!-- None. Belts keep their behavior; the handoff interface gains a vertical direction used only by vertical conveyors. -->

## Impact

- **Core:** `FactoryKind::VerticalConveyor` with `{ up: bool }` and the belt slot, a vertical step in the belt system, and `Facing`-free handoff for vertical moves (`insert_from` with a heading of `None` up or down treated as entering from below or above).
- **Client:** glyphs for up and down following `docs/ART-DIRECTION.md` (CP437 `▲` / `▼` in the belt color, so they differ from belts' `^` / `v` and from extractor arrows), and the look panel showing the target layer.
- **Showcase:** an example that lifts items from z = 0 to a chest on z = 1.
