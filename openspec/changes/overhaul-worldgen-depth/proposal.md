# Proposal

## Why

The world is meant to get stranger and richer the deeper the player digs, and the factory tiers lean on that: vertical conveyors and stairs for 3D factories, ore spouts underground (`add-ore-spouts`), ruined reactors (`add-ruined-reactors`) and nuclear slimes (`add-nuclear-slime`) in the deep. A survey of seed 12345 (512 by 512 tiles per layer) shows the generator doesn't support that yet:

| Layers | What generates |
|---|---|
| z = +5 to -5 | The same grassy plains on every layer: about 74% grass, 16% rock, 4% air, trees. There is no sky above the surface and no rock just below it. |
| z = -10 and deeper | Suddenly 86% rock and 14% open caves, the same at every depth. |
| Deposits, any depth | Iron scrap about 0.02% of tiles; scrap electronics and plasteel round to 0.00% (at most one tile per 64 by 64 chunk). |

Problems beyond the numbers:
- **Deposits that don't work:**
  - Plasteel scrap generates but has no mining drop, so it can't be mined.
  - Iron scrap and plasteel scrap are walkable, unlike rock.
  - Scrap electronics, needed for the belt router and the auto-harvester, is so rare that a run can stall.
- **No connected underground:**
  - Each layer is an independent noise slice, so caves on neighboring layers don't line up.
  - Nothing generates stairs between layers.
  - A new game starts in the SapienCorp bunker at z = -19 with the quest "Escape the dark bunker". The bunker has a staircase, but nothing outside it guarantees a route the rest of the way to z = 0 except digging stairs by hand.
- **The docs are stale:** `docs/WORLDGEN-ARCHITECTURE.md` describes a z formula and a fluid manager that no longer match the code.

## What Changes

- **A vertical layout with named bands**, from a single `depth_band(z)` function that worldgen, spawning and features all use:
  - **Sky** (z ≥ 1): open air. Players build up into it with stairs and vertical conveyors.
  - **Surface** (z = 0): today's biomes, generated exactly as now, so existing scripts, snapshots and the spawn area don't change.
  - **Shallows** (z = -1 to about -8): dirt and rock with tunnels, roots near the top, and the first deposits.
  - **Deep** (below the shallows): the Lithic Rivers caverns, getting more open and richer with depth.
- **Caves that connect across layers:** cave noise sampled with a small z scale, so a cavern on one layer mostly lines up with the next.
- **Natural stairways:** where caves on two neighboring layers overlap, a seeded few become stairways, so the underground can be walked through. A guaranteed route connects the bunker to the surface.
- **Deposits as clusters with a depth table:** each resource has a vein-noise field and a density per band (iron from the shallows down, scrap electronics in the shallows and deep, plasteel in the deep).
  - Every deposit tile is solid and mineable with a drop. Plasteel gets an item, or is removed until it has a use.
  - Tuned so a player finds enough scrap electronics for the early factory within a few minutes of digging.
- **A per-chunk feature stage** after terrain, with a deterministic order and its own seeded generator. Deposits, natural stairs, creature spawns (slimes, by band), and later ore spouts and ruined reactors all place through it, so each feature can't shift another's rolls.
- **A worldgen survey:**
  - A test prints and pins each band's tile mix and deposit counts for a fixed seed, so tuning is deliberate and visible in review.
  - The ore-test debug world shows the bands and deposits.
- **Old worlds keep their generator:**
  - A `worldgen_version` in the world settings. Saves without it use today's generator, so already-explored worlds don't get seams.
  - New worlds use version 2.
- **Docs:** `docs/WORLDGEN-ARCHITECTURE.md` rewritten to match.

## Capabilities

### New Capabilities
- `worldgen-depth`: Depth bands, connected caves, natural stairways, deposit tables, the feature stage, the survey and generator versions.

### Modified Capabilities
<!-- None in openspec/specs yet: worldgen has no spec today. debug-worlds keeps its world types; the ore-test world gains bands. -->

## Impact

- **Core:**
  - `world.rs` generation split into terrain (by band) and a feature stage.
  - `depth_band`, deposit tables and the natural-stair pass.
  - `WorldSettings::worldgen_version` (serde default 1 for old saves).
  - Mining drops for every deposit tile, and passability fixes in `tile_registry.rs`.
- **Other changes:**
  - `add-nuclear-slime` spawns by band through the feature stage.
  - `add-ore-spouts` and `add-ruined-reactors` place through it.
- **Tests:**
  - The survey's pinned bands.
  - The bunker-to-surface route.
  - Surface (z = 0) generation unchanged: the existing golden entity test and every snapshot stay as they are.
  - Old saves loading with version 1.
