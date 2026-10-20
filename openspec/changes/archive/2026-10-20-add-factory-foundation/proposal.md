# Proposal

## Why

LithicRivers is heading toward automation (tree farms, conveyor belts, a power grid; see `add-tree-farming`, `add-conveyor-belts`, `add-power-grid`, `add-tree-automation`). None of that fits today's world model: placed blocks are plain `TileKind` values with no direction, inventory or state, and the world only advances when the player acts, one system pass per action. This change adds the shared foundation every factory feature builds on.

## What Changes

- **Factory blocks**: placeable blocks backed by an ECS entity on their tile (a `FactoryBlock` with a kind and a facing direction), so they can hold state, inventories and progress. Breaking one returns its item and spills its contents.
- **Facing and rotation**: rotate a block before placing it (a rotate key in Place mode, with the current facing shown in the hotbar), and after placing it (a new **Rotate** build mode: direction keys rotate the adjacent block clockwise).
- **Chests**: the first factory block, a storage block with an inventory, opened with Interact into a transfer window (reusing the corpse-looting window).
- **Factory time**: factory systems advance in fixed steps by elapsed game ticks, so a long action runs more steps than a short one; 100 ticks = 1 game second. Factory blocks simulate wherever they are placed, not only near the player.
- **Pass time**: a new keybind passes 10 game seconds so the player can let the factory run; it stops early if combat starts.
- **Chunk boundaries**: factory blocks work identically across chunk edges, including negative coordinates and chunks the player has never visited.

## Capabilities

### New Capabilities
- `factory-blocks`: Placeable stateful blocks with a facing, rotation before and after placing, breaking, chests, and behavior across chunk boundaries.
- `factory-time`: How factory systems advance with game time, where they simulate, and passing time.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `FactoryBlock` / `Facing` / `Storage` components, a factory step scheduler fed by elapsed ticks, block placement/break/rotate functions, chunk-safe neighbor access.
- Client: Rotate build mode, rotate-before-place key, chest transfer window, pass-time key, factory glyphs in the world view.
- Config: `build.ROTATE_PLACEMENT`, `action.PASS_TIME`.
- Every new component must be registered for saving once `flawless-world-saves` lands (its completeness test enforces this).
- Crafting recipes for chests come from `add-crafting-tree`.
