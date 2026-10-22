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
- *Changed during implementation:* two kinds, `FactoryKind::VerticalConveyorUp` and `VerticalConveyorDown`, each with a `BeltSlot`, instead of one kind with a `VerticalConveyor { up }` component. Both break into the one `VerticalConveyor` item, and the item places an up conveyor. With the direction in the kind, saves, blueprints and sprites keep it with no new component.
- It isn't directional in x/y, so Rotate swaps it between the two kinds (`RotateResult::Flipped`) instead of turning a facing.
- It carries items, not the player: standing on one doesn't ride it.

### Movement
- After `BELT_STEPS_PER_TILE` steps, the item is offered to the block at z ± 1, with `heading: None`.
- Chests and machines accept as they do from the player.
- A belt above accepts only while empty. A vertical conveyor above, going the same way, accepts too, so stacks work.
- Vertical conveyors join `belt_order` as nodes whose next node is the block above or below, keeping one-step line shifts across layers.

### Open target
- *Simplified during implementation:* items are only handed to a factory block, so a conveyor under rock, open air or an unloaded chunk just holds its item. Nothing needs generating, and nothing is dropped.

### Recipes
- **Vertical Conveyor:** the same ingredients as Wooden Conveyor, giving 2.
- **Conversions:** 1 Wooden Conveyor to 1 Vertical Conveyor, and 1 Vertical Conveyor to 1 Wooden Conveyor. The reachability test is unaffected by the cycle.

## Risks / Trade-offs

- [Belt order cycles through layers] → `belt_order` already handles loops by position; vertical edges use the same cycle guard.
- [Players can't see the other layer] → The look panel names the target layer and what is there ("Lifts to z = 1:" and the block, or "Rock, blocked").
- The showcase example goes last in `EXAMPLES`, so the examples before it keep their coordinates (scripts and demos teleport to them).
