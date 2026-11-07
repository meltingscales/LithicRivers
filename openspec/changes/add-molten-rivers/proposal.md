# Proposal

## Why

The deep band is named after the Lithic Rivers, but its river channels are empty air today. The Steam MVP asks for an underground biome of "molten lava and rare ore" (`docs/STEAM-MVP.md`, core gameplay loop). Without lava the deep looks like the shallows with wider caves, and without a material found only there, the player has no reason to go down once they have plasteel. This change fills the rivers with lava and puts a rare ore on their banks, so the deep looks and plays like its own place and the late tier gets a material of its own.

Mining bots, the third part of that MVP line, are a separate change (`add-mining-bots`), so this one stays about terrain and materials.

## What Changes

- **Lava rivers:**
  - In the deep band, the core of each river channel is molten lava instead of air.
  - Crust fords cross each river where a slow noise field says so, so the deep stays explorable without building anything.
  - Lava can't be walked through by the player or by creatures, can't be mined, and doesn't block sight.
- **Bridging:** placing a plain block (stone, plank block, stone brick) on lava sinks it and leaves a walkable lava-crust tile, so the player can make their own crossings at the cost of a block.
- **Riverglass:** a new rare deposit that forms only in the rock along the rivers' rims in the deep band. It mines into a new item, riverglass.
- **Riverglass in recipes:** the teleporter and the advanced power cell now need riverglass. The teleporter recipe was a placeholder waiting for a late-tier material.
- **Art:** sprites for lava, lava crust, the riverglass tile and the riverglass item, following `docs/ART-DIRECTION.md`.

## Capabilities

### New Capabilities
- `lava`: What lava does to movement, mining, sight and block placement.

### Modified Capabilities
- `worldgen-depth`: The deep band's rivers carry lava with fords, and riverglass deposits form on their rims.

## Impact

- **Core:** new `TileKind`s `Lava`, `LavaCrust` and `Riverglass` and new `ItemKind::Riverglass` (appended, so the order of existing items stays the same); the deep-band terrain in `world.rs`; a mining drop; a shared plain-block placement helper in `factory.rs` that the client calls; `blocks_sight` lets sight pass over lava; recipe changes.
- **Client:** sprites; the block placement path calls the core helper.
- **Saves:** chunks explored before this change keep their old empty rivers; chunks explored afterwards get lava. Seams are accepted, as in `worldgen-depth`.
- **Surface:** unchanged. `the_surface_is_generated_as_before_depth_bands` keeps passing.
- **Snapshots:** the deep-band snapshot, the Item Dictionary, the filter picker and the crafting views that list the changed recipes are expected to change.
