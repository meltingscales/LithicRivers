# Proposal

## Why

The enemy-spawning playtest passed ("hard but fair"), but natural spawns felt like a bit much. The user also wants cleared areas to stay cleared after one refill, so that a steady supply of creatures (meat, embryos) comes from spawners the player builds rather than from the wild.

## What Changes

- **Half the natural spawn chances:** feral dogs 15% to 8% by day and 30% to 15% at night; nuclear slimes 10% to 5% in the Shallows and 20% to 10% in the Deep. Caps, distances, the light and fence rules, and spawner blocks are unchanged.
- **Each chunk spawns naturally at most once:** after a creature spawns naturally in a chunk, that chunk never spawns one naturally again. This replaces the 5-minute rest. A cleared area refills once and then stays empty, apart from creatures that walk in and spawner blocks.
- The set of spent chunks is saved with the world. Saves from before this change load with no chunk spent, so each chunk may spawn once more.

Out of scope: wild spawners in ruins and placed spawners (unchanged), creatures placed at worldgen, and any per-kind or per-biome limits beyond the existing caps.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `enemy-spawning`: "Areas refill within limits" becomes "Areas refill once, within limits".

## Impact

- `crates/core/src/systems/creature_spawn.rs`: the rule chances, and a spent-chunk check in place of the cooldown.
- `crates/core/src/resources.rs`: `spawn_cooldowns` becomes `spawned_chunks` (a saved set).
- `AGENTS.md`: the note about creatures respawning on their own.
