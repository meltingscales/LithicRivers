# Design

## Context

- **Belts:** `BeltSlot { item, progress }`, processed in `belt_order` so a full line shifts in one step, handing off through `insert_from(target, item, Some(facing))`.
- **The factory index** is keyed by full `Position`, so blocks on any z-level are already found.
- **Rotate mode** turns directional blocks clockwise.

## Goals / Non-Goals

**Goals:** move items between layers with the same feel as belts; deterministic; no change to flat belts.

**Non-Goals:** lifting the player (stairs do that), lifts for fluids.

## Decisions

### Block
- `FactoryKind::VerticalConveyor` carries a `BeltSlot` like a belt, plus `VerticalConveyor { up: bool }`.
- It isn't directional in x/y, so Rotate toggles `up` instead of turning a facing.

### Movement
- After `BELT_STEPS_PER_TILE` steps, the item is offered to the block at z ± 1, with `heading: None`.
- Chests and machines accept as they do from the player.
- A belt above accepts only while empty. A vertical conveyor above, going the same way, accepts too, so stacks work.
- Vertical conveyors join `belt_order` as nodes whose next node is the block above or below, keeping one-step line shifts across layers.

### Open target
- The target tile must be passable or a factory block.
- If it's solid or out of loaded chunks, the conveyor first generates the chunk (through `Game::factory_tile` semantics) and then holds its item if the tile is solid.

### Recipes
- **Vertical Conveyor:** the same ingredients as Wooden Conveyor, giving 2.
- **Conversions:** 1 Wooden Conveyor to 1 Vertical Conveyor, and 1 Vertical Conveyor to 1 Wooden Conveyor. The reachability test is unaffected by the cycle.

## Risks / Trade-offs

- [Belt order cycles through layers] → `belt_order` already handles loops by position; vertical edges use the same cycle guard.
- [Players can't see the other layer] → The look panel names the target layer and what is there.
