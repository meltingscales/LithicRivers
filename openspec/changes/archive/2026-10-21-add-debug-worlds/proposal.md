# Proposal

## Why

Testing a mechanic in a normal world means finding the right place first: an open area to build in, a vein of every ore, a crowd of enemies, a working factory to compare against. That's slow for developers, and slow for players who want to learn a machine before using it for real. Minecraft solves this with world types ("superflat", debug worlds). LithicRivers needs its own set, chosen when a world is created.

## What Changes

- **A World type option** on the Create New World screen, shown when Developer mode is on.
  - Choices: **Normal** (default), **Void**, **Ore test**, **Dog arena** and **Factory showcase**.
  - The seed still applies, so every debug world is reproducible.
- **Void:**
  - Empty space with a small grass platform at the spawn point.
  - No terrain, structures, creatures or spouts.
  - For building and testing machines in isolation.
- **Ore test:**
  - A flat area at spawn with labeled rows of every minable tile: rock, iron scrap, scrap electronics, and, once they exist, ore spouts and reactor parts.
  - Built from the mining drop table, so new materials appear automatically.
- **Dog arena:** a normal world with 100 feral dogs placed in rings around the spawn point, for combat and performance testing.
- **Factory showcase:**
  - A flat area with prebuilt example setups, arranged from simple to complex: a chest, a belt line into a chest, a furnace fed by belts, chutes and funnels, and simple, moderate and complex production chains, as those machines exist.
  - Each later factory change adds its examples to the showcase.
  - A test checks that every factory block kind appears in at least one example.
- **The world list shows the type** for non-normal worlds, for example "(Void)".

## Capabilities

### New Capabilities
- `debug-worlds`: The world types, what each one generates, when the option is offered, and the showcase coverage rule.

### Modified Capabilities
<!-- None. The world-saves "Create a world" requirement is unchanged for Normal worlds; the option is additive and stored with the world's settings. -->

## Impact

- **Core:**
  - `WorldSettings` gains `world_type: WorldType`, saved with the world and in `meta.json`, with a serde default of `Normal` for existing worlds.
  - `GameWorld` chunk generation branches on the type: void chunks, and a flat area for ore test and showcase.
  - `Game::new` gains a world-type-aware setup that places ore rows, dogs and showcase builds deterministically.
  - A `showcase` module holds the example builds as data.
- **Client:**
  - The World type row in Create New World, visible with Developer mode on.
  - The type label in the world list.
- **Tests:** generation tests per type, and the showcase coverage test.
- **Dependencies:**
  - Uses factory blocks from `add-factory-foundation` (chests).
  - Showcase examples grow with `add-conveyor-belts`, the furnace changes and later changes, each of which adds its examples and must keep the coverage test passing.
