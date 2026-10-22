# Proposal

## Why

Creatures are only placed once: feral dogs at world creation, and nuclear slimes once per underground chunk (`creature_spawn_system`). Kill them and an area stays empty forever, so combat, the creature loot table and the nuclear waste drop all run dry. The user's idea (IDEAS-INBOX.md): creatures that keep appearing where they belong, with limits so they never flood, and enemy farming as something worth building for.

## What Changes

- **Spawn rules by place:** a table of where each creature keeps appearing, checked as the player explores:
  - nuclear slimes in the deep, and more near nuclear waste once `add-ruined-reactors` places it
  - feral dogs on the surface in forests, and at night more often (`add-day-night-cycle`)
- **Limits:**
  - At most a set number of each creature within a radius of the player (for example 6 dogs within 48 tiles).
  - A cooldown per chunk after a spawn.
  - Never in view, never within 16 tiles of the player, never on factory blocks or inside lit, enclosed bases (a lamp or a fully fenced area stops natural spawns).
- **Spawner blocks:** found in ruins and dungeons, each spawns one creature kind nearby on a timer while the player is within range, up to its own cap.
  - Breaking one drops its core: a "spawner core" item that can be placed again.
  - This is how farms are built: wall in a spawner, let creatures come to a killing spot.
- **Fence:** a craftable block that blocks creatures but not sight or light. The player can walk through a fence gate; fences themselves block the player too.
- **Encouraging farming:** spawner farms and creature loot go through the same loot tables, and creatures killed by machines drop into the machine's output once machine traps exist (later).

## Capabilities

### New Capabilities
- `enemy-spawning`: Spawn rules and limits, spawner blocks, and fences.

### Modified Capabilities
<!-- None in openspec/specs: creature spawning has no spec yet (add-nuclear-slime adds one). -->

## Impact

- **Core:**
  - A `SpawnRule { creature, band, biome or tile condition, chance, cap, radius }` table.
  - `creature_spawn_system` runs the rules on a slow timer with per-chunk cooldowns (saved).
  - `FactoryKind::Spawner` (with its creature and timer) and the spawner core item.
  - `TileKind` or `FactoryKind` for the fence and fence gate.
  - Creature movement treats fences as walls.
- **Client:** sprites for spawners and fences, and the look panel naming a spawner's creature and timer.
- **Showcase:** a fenced dog-spawner farm.
- **Depends on:**
  - `add-nuclear-slime` (`creature_spawn_system`)
  - `overhaul-worldgen-depth` (bands)
  - lights and night from `add-day-night-cycle`, for the lit-base rule

## Open questions

- Fences: should the player pass through plain fences too (only creatures blocked), or only through gates?
- Should spawners be craftable later, or only found?
