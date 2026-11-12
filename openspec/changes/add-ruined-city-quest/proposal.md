# Proposal

## Why

The story stops after "A New Arm": the player has no goal and learns nothing more about the reactor. A third quest should keep the lore coming, give the new guns, androids and power grid a reason to be used, and keep sending the player back to the repaired android at the start. The surface also has only plains, forests and rocky ground, with nothing that says an advanced world was lost here; a ruined city does.

## What Changes

- **A new surface biome, the Ruined City:**
  - Rubble, ash and cracked roads in place of grass and trees, decided by its own noise so cities are rare and large, and the rest of the surface keeps its look.
  - **Skyscrapers:** multi-floor ruin structures with stairs and collapsed floors, holding scrap, electronics and a military android or two. They appear in city chunks the way outposts appear in forests.
  - New tiles: ash and cracked road (walkable), rubble (mineable into stone and sometimes iron scrap) and concrete (the skyscrapers' walls).
- **The SapienCorp datacenter:** a guaranteed quest structure, one per world, at a fixed place about half a mile from the bunker, with the Ruined City forced around it so it always stands in a city.
  - Dark and sealed. Its server hall is behind a **sealed door**, a new factory block that opens only while its power network is powered. The datacenter has a burner generator with no fuel, so the player brings fuel (or builds their own power).
  - Guarded by military androids, with a data shard in the server hall's quest chest and a dormant transit station (the "Datacenter Station").
- **Quest 3, "Black Box":** offered by the repaired tutorial android (SC-M4X7) once "A New Arm" is complete.
  1. Reach the datacenter.
  2. Restore its power (the sealed door opens).
  3. Take the data shard from the server hall.
  4. Bring it back to SC-M4X7.
- **Reward** when the shard is handed in: a standard fusion core, and one gun with ammo, picked from a list of mid-tier guns by the world seed, so each world has its own and the same world always gives the same one.
- **Lore:** the shard's description and SC-M4X7's dialogue reveal that the reactor was pushed past its limits on purpose, under orders from SapienCorp's chief executive and board, for a project whose name stays redacted, run by one named project director. It points toward the next quest without saying what the project was.

## Capabilities

### New Capabilities
- `ruined-city-biome`: where the Ruined City appears, its tiles, and its skyscraper ruins.
- `black-box-quest`: the datacenter, the sealed door, the quest's steps, the reward and the lore it reveals.

### Modified Capabilities
None. The arm quest and worldgen bands keep their requirements; this adds a surface biome without changing the depth bands.

## Impact

- **Core:**
  - `BiomeBand::RuinedCity`, a city noise field, the forced city around the datacenter, the new tiles and the skyscraper structure roll.
  - `QuestType::BlackBox`, its objectives (a new "power restored at a place" objective), dialogue nodes, hand-in and the seeded reward.
  - `FactoryKind::SealedDoor` (a powered consumer that is passable only while satisfied).
  - The datacenter placed like the facility, with its marker, quest chest contents and transit station name.
  - The surface hash pin changes on purpose (cities appear in the surveyed area).
- **Content:** `datacenter` and `skyscraper` structures as `plan.toml`, checked with `just util-structure-preview`; the data shard item with its lore text; sprites for the tiles, door and items.
- **Client:** the Quests tab and markers already show new quests; `tp_quest` gains `datacenter`.
- **Depends on:** military androids (`add-ranged-combat`), the power grid, transit stations and firearms (all merged).
