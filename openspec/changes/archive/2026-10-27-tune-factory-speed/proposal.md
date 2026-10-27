# Proposal

## Why

Playtesting the electric furnace showed factories run far too fast. Game time only passes when the player acts, and one step costs 2 game seconds, so in one step a belt carries an item 4 tiles, an extractor moves 2 items and a furnace gets halfway through a log. Factories outpaced the player instead of being something to build and watch.

## What Changes

- **Factory seconds:** every factory rate and power figure is now per factory second, and a factory second lasts 3 game seconds. The factory clock steps every 60 ticks instead of 20, so every block slows by the same factor and the ratios between blocks (power budgets, belt throughput, fuel per smelt) are unchanged.
- **Belt riding stays quick:** a belt carries the player one tile every factory step (about 1.7 tiles per game second; it was 2), instead of slowing down with items.
- Tree growth and Pass Time stay in game time.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `factory-time`: factory seconds, 3 game seconds each.
- `conveyor-belts`, `belt-logistics`, `tree-automation`: rates stated in factory seconds; the player rides a belt one tile per factory step.

## Impact

- **Core:** `FACTORY_SECOND_TICKS` and `STEPS_PER_FACTORY_SECOND` in `factory.rs` (renamed from `power::STEPS_PER_SECOND`); `FACTORY_STEP_TICKS` derived from them; `BELT_RIDE_STEPS_PER_TILE` for the player.
- **Scripts and snapshots:** scripts that ticked less than one step, or waited for saplings, are re-timed; showcase snapshots show slower progress.
