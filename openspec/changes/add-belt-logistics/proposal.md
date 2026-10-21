# Proposal

## Why

Belts can only carry what lands on them: a machine output, a dropped item or one the player puts on. Nothing takes items out of a chest, nothing sorts a mixed belt, and lines can't split. So the tree farm can't feed its own acorns back to the planter, and a generator can't be fed logs from a chest. The player also can't walk across a belt line, which makes a base a maze. Factorio's inserters, filter inserters and splitters, and its walkable belts, solve these.

## What Changes

- **Extractor:** a directional block that takes one item at a time from the block behind it (a chest, a machine output or a generator's fuel) and puts it into the block in front (a belt, a chest or a machine). It is unpowered and slow.
- **Filter extractor:** an extractor that only moves the one item kind set in its window.
- **Belt router:** a block that sends items arriving on a belt to its other outputs in turn (left, straight, right), skipping blocked ones; an optional filter sends one item kind one way and everything else straight on.
- **Walking on belts:** belts become walkable, and a belt carries a player standing on it one tile in its facing direction at belt speed, if the next tile is free. Other factory blocks stay solid.
- **Recipes** for the new blocks, and **showcase examples** for each (required by the showcase coverage test).

## Capabilities

### New Capabilities
- `belt-logistics`: Extractors, filter extractors, belt routers, and walking on belts.

### Modified Capabilities
<!-- None. Conveyor belt movement is unchanged; walking on belts is added here. -->

## Impact

- **Core:** new `FactoryKind`s (`Extractor`, `FilterExtractor`, `BeltRouter`), a take-one-item interface alongside `factory::accepts` / `insert`, a factory step for extractors and routers (after machines, before belts), and passability that depends on the factory block kind (belts passable, others solid).
- **Client:** glyphs, the filter picker window, and the help text.
- **Showcase:** an example per new block, including a self-feeding tree farm that sends acorns back to its planter.
- **Depends on** conveyor belts, tree automation and debug worlds (the showcase).
