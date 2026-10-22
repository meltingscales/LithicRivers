# Proposal

## Why

The world is large and its interesting places are rare: Fractal Tears (`add-fractal-tear-biome`), ruined reactors, and the new surface biomes. Wandering until one turns up is no fun. The user's idea (IDEAS-INBOX.md): a moderately expensive biome compass, powered by a power cell (`add-power-cells`), that finds the nearest biome of a chosen kind and can mark it on the Global Map.

## What Changes

- **The biome compass:** a crafted, mid-game gear item (scrap electronics, iron scrap and a crude power cell to build).
- **Using it:**
  - Pick a biome kind from a list: the surface biomes, the Lithic Rivers depths, the Fractal Tear, and others as they're added.
  - The compass reports the direction and distance to the nearest one ("Fractal Tear: 3,420 tiles south-east, 40 layers down"), using charge from a carried power cell.
  - Choosing "Mark it" adds a waypoint on the Global Map at that spot.
- **Search range:** up to 8,000 tiles. Beyond that it says "None within range". Each search costs a fixed charge, and a search that finds nothing costs the same.
- **Biome kinds the compass knows** come from one list, so a biome added later shows up automatically.

## Capabilities

### New Capabilities
- `biome-compass`: Searching for the nearest biome of a chosen kind, reporting direction and distance, and marking it on the Global Map.

### Modified Capabilities
<!-- None: Global Map markers already exist (quest markers). -->

## Impact

- **Core:**
  - `world::nearest_biome(kind, from, range)`: a search over the biome functions without generating chunks (biomes are pure functions of position).
  - The compass item and recipe.
  - A `QuestMarkerType::Waypoint` marker.
- **Client:** the compass window (biome list, result, mark button), and the waypoint on the Global Map.
- **Depends on:**
  - `add-power-cells` (charge)
  - `overhaul-worldgen-depth`'s bands
  - `add-fractal-tear-biome` and future biome changes, for kinds to find
