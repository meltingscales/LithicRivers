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
- **Spawner blocks:** wild spawners sit in ruins and dungeons, each spawning one creature kind nearby on a timer while the player is within range, up to its own cap, with nothing to feed them.
  - Breaking one gives a spawner block item. Spawners are also craftable (decided by the user).
  - A spawner the player places runs only with power, a supply of **meat slurry**, and an **embryo** that sets which creature it grows. Embryos come in three grades (poor, flawed, flawless); a better embryo lasts for more spawns.
  - Creatures drop embryos of their own kind now and then, poor most often, so a farm can feed itself if the player automates it: belts bring slurry and carry embryos back.
  - This is how farms are built: wall in a spawner, keep it fed, let creatures come to a killing spot.
- **Fences and gates** (decided by the user): craftable blocks that don't block sight or light.
  - **Fences** block creatures and the player.
  - **Gates** block creatures, but the player walks straight through them, with nothing to open or close.
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
  - `FactoryKind::Spawner` (wild with a fixed creature, or placed with slurry and embryo slots), the spawner item and recipe.
  - Items: meat slurry (crafted from meat), and an embryo per creature kind and grade; embryo drops in the creature loot tables.
  - `TileKind` or `FactoryKind` for the fence and fence gate.
  - Creature movement treats fences as walls.
- **Client:** sprites for spawners and fences, and the look panel naming a spawner's creature and timer.
- **Showcase:** a fenced dog-spawner farm.
- **Depends on:**
  - `add-nuclear-slime` (`creature_spawn_system`)
  - `overhaul-worldgen-depth` (bands)
  - lights and night from `add-day-night-cycle`, for the lit-base rule
  - the power grid (`power-grid` spec), for placed spawners

## Decided

- **Fences and gates (the user's decision):** the player walks through gates only, without opening or closing them, and not through fences.

- **Craftable spawners (the user's decision):** spawners can be crafted, but a placed spawner needs power, meat slurry and an embryo (poor, flawed or flawless), so monster farming is tied to automation.
