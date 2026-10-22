# Design

## Context

- **Biomes are pure functions of position:**
  - `GameWorld::biome_for(x, y, z)` for surface bands
  - `depth_band(z)` for depth
  - Fractal Tear regions decided by `feature_rng`
- **Global Map markers** are `QuestMarker`s.

## Goals / Non-Goals

**Goals:**
- A fast, deterministic search that doesn't generate chunks.
- One list of findable biomes.
- A clear result.

**Non-Goals:**
- A live needle that points while walking (the result is a reading plus a waypoint).
- Finding structures (a later compass mode).

## Decisions

### Search
- The search samples the biome function on a grid that widens outward in rings: every 16 tiles near the robot, every 64 tiles past 2,000 tiles. It stops at the first ring with a match, then refines to the nearest matching sample in that ring.
- **Depth-only kinds** (the Lithic Rivers depths) report the nearest layer where they start, below the robot.
- **Region-based kinds** (Fractal Tears) test regions directly.
- **Cost:** a search is bounded (at most 8,000 tiles), takes no game time, and costs 50 kJ from a cell.

### Findable biomes
`BiomeKind::ALL`, with a name and a test function each. The compass lists it, and a test fails if a biome added to worldgen isn't in it.

## Risks / Trade-offs

- [Sampling misses small biomes] → The grid spacing near the robot is small, and biomes smaller than 16 tiles aren't worth a compass.
