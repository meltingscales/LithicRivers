# Design

## Context

- Belts: `BeltSlot`, `belt_order` (each belt after the one it feeds, with a cycle guard) and `step_belts`, handing off through `insert_from(target, item, heading)`.
- Vertical conveyors set the pattern: the block's role is its `FactoryKind`, and `belt_target` decides where a belt-like block hands its item.
- Power poles already link blocks and draw blinking lines between them (wires, `power.links`).

## Goals / Non-Goals

**Goals:** cross belt lines on one layer; the same item rate as belts; nothing lost when an end breaks; deterministic.

**Non-Goals:** pipes and fluids, different tiers or ranges, crossing between layers (vertical conveyors do that).

## Decisions

### Linking
- On placement, an underground conveyor looks behind it, against its facing, for up to 10 tiles.
  - If the nearest unlinked entrance facing the same way is there, the new block becomes that entrance's exit.
  - Otherwise it becomes an entrance.
- Links are stored on both ends and rebuilt by position on load if missing.

### Transit
- The entrance's `BeltSlot` takes items as a belt does. After `BELT_STEPS_PER_TILE` steps, the item enters the tunnel with `distance * BELT_STEPS_PER_TILE` steps to go, at most one item per tile of tunnel.
- The exit holds one item in its own `BeltSlot` and hands it on along its facing. A full exit backs the tunnel up, which backs up the entrance.
- `belt_order` gets an edge from entrance to exit, so a line through a tunnel still shifts in one step.

### Breaking
- Breaking either end gives its item back and drops everything in transit on the entrance's tile. The other end becomes unlinked: an entrance with no exit holds its item, and an exit with no entrance does nothing.

## Risks / Trade-offs

- [Players can't see the tunnel] → A blinking line along the tunnel, on even seconds like wires, and the look panel naming the other end and the distance.
- [Linking to the wrong entrance in a crowded row] → Only an unlinked entrance facing the same way links, and the nearest wins, as in Factorio.
