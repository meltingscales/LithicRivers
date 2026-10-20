# Proposal

## Why

Trees are the only renewable resource, and they don't renew: chopping a tree turns it into dirt for good, and the acorns it drops can only be crafted into wood shavings. A sustainable wood supply is the first step of the automation loop (plant → grow → harvest → belt → chest → fuel and crafting).

## What Changes

- **Planting**: the player can plant an acorn on grass or dirt, creating a sapling.
- **Growth**: a sapling grows into a tree after a deterministic number of game minutes (based on seed and position), advancing with factory time, wherever it is.
- **Saplings**: shown with their own glyph; walking through them is allowed; breaking one returns the acorn.
- **Grown trees** are the same `Tree` tile as natural trees, so chopping and drops work unchanged.

## Capabilities

### New Capabilities
- `tree-growth`: Planting acorns, sapling growth over game time, and saplings as world objects.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `TileKind::Sapling`, a `Growth` component (planted tick, grow-at tick) on a sapling entity, a growth factory system, planting via the existing Place flow.
- Client: sapling glyph and sprite, planting through the hotbar (acorn becomes placeable onto grass/dirt).
- Depends on `add-factory-foundation` (factory time, simulate wherever placed). `add-tree-automation` builds auto-planters and auto-harvesters on this.
