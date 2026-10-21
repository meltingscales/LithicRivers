# Proposal

## Why

Machines need a way to move what they produce. A primitive wooden conveyor belt that carries items one tile at a time into a chest is the simplest logistics that makes an automated tree farm (harvester → belt → chest) possible.

## What Changes

- **Wooden conveyor**: a directional factory block that carries items in its facing direction, one item per belt tile, moving one tile every half game second.
- **Handoff**: a belt pushes its item into whatever it faces: the next belt, a chest, or a machine's input. If the target is full or blocked, items queue up behind it.
- **Loading belts**: items dropped on a belt tile are picked up by the belt; the player can also drop items onto belts. Machines from later changes output onto belts.
- **Belts across chunk edges** work like anywhere else.
- **Recipe**: wooden conveyors are craftable (planks, sticks, string).

## Capabilities

### New Capabilities
- `conveyor-belts`: Belts carrying items, handing them to chests, machines and other belts, and loading items onto belts.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `FactoryKind::Belt`, a `BeltSlot { item, progress }` component, the belt factory system (runs after machines), a generic `accepts_item` / `insert_item` interface for chests and machines.
- Client: belt glyphs (`^ > v <`) with the carried item shown on the tile.
- Depends on `add-factory-foundation` and `add-crafting-tree`.
