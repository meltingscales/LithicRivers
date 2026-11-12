# Design

## Context

- **Surface generation** (`world.rs`): `generate_surface` builds tiles from the legacy noise per biome (`biome_for` picks Plains, Forest or Rocky from one Perlin field), then adds tree clusters, rare resources and wild crops. `worldgen_tests::the_surface_is_generated_as_before_depth_bands` pins a hash of 25 surface chunks.
- **Random structures:** each chunk rolls 5% for one structure chosen by its biome (outposts in forests, temples on plains, dig sites in the deep).
- **Quest structures:** the facility is queued as a `PendingStructure` at the fixed `FACILITY_POS` in `Game::new`, with a quest marker; its `TreasureQuest1` tiles become quest chests (`place_quest_chests`). Transit stations take their names from the structure (`teleporter::station_name`).
- **Quests** (`quests.rs`, `dialogue.rs`): `QuestType`, ordered objectives checked every tick by `quests::advance` (`GoToLocation`, `FetchItem`, `InstallPart`), and dialogue nodes that unlock quests (`unlocks_quest`, `requires_quest_complete`, `requires_item`).
- **Gates** are factory blocks the player walks through and creatures don't.

## Goals / Non-Goals

**Goals:**
- A city that reads as the old world's ruins, without changing the rest of the surface.
- A guaranteed, reachable third quest that uses power, guns and androids, and returns the player to the start.
- A lore reveal with names, and a hook for a fourth quest.

**Non-Goals:**
- The fourth quest itself, or a marker for it.
- Other new biomes (frozen wastes, arboreal, and the rest stay in `docs/SPEC-GAPS.md`).
- City-specific creatures; dogs and androids are enough for now.

## Decisions

### The city is its own noise, layered over the existing biomes
`biome_for` first checks a city field (its own Perlin, large scale, high threshold): above it, the tile is Ruined City; otherwise the existing three-way pick runs unchanged. Within 96 tiles of the datacenter the city is forced. Alternative: a fourth center in the existing biome field. Rejected: it would move every biome border and change most of the surface. The hash pin still changes where a city falls in the surveyed area, which is re-pinned on purpose.

City tiles come from a small generator per tile: mostly ash, a road grid every 16 tiles (cracked road), and rubble patches from feature noise. Tree clusters, rare resources and wild crops skip city tiles.

### Skyscrapers ride the per-biome structure roll
`BiomeBand::RuinedCity` gets the skyscraper in the existing 5% chunk roll, raised to 30% for city chunks so a city has a skyline. The skyscraper is a `plan.toml` of four floors (layers) with stairs, concrete walls, collapsed floor tiles (air holes), scrap and treasure tiles, and an `android_spawn` on an upper floor.

### The datacenter is placed like the facility
`DATACENTER_POS` is fixed at (-620, 380, 1), about 0.45 miles from the bunker, queued in `Game::new` as a pending structure (not buried, it stands on the surface), with its marker added when the quest starts. Its plan holds:
- an entrance hall with two `android_spawn` tiles,
- the server hall behind a sealed door, holding a `TreasureQuest2` tile that becomes a quest chest with the data shard,
- a generator room with a burner generator and a pole (structure blocks, like the stations' RTG) and no fuel,
- a `teleporter_station` tile, named "Datacenter Station" by `station_name`.

`TreasureQuest2` is a new tile (appended to `define_tiles!` to keep the hash of existing tiles), and `place_quest_chests` gives each quest tile its item: the arm for `TreasureQuest1`, the data shard for `TreasureQuest2`.

### The sealed door
`FactoryKind::SealedDoor`: a power consumer drawing 10 kW, always active. It is passable for the player (and only the player, like a gate) while its consumer's satisfaction is above zero, and solid otherwise; the look panel and sprite show open or sealed. Placed by the structure as a structure block. Alternative: a door opened by a key item. Rejected: power is the mechanic this quest should teach.

### Objectives
`QuestObjectiveType::PoweredAt { x, y, z }`: met while the factory block at that position has a satisfied consumer. `quests::advance` passes the world and resources into the objective check for it. Because objectives never come undone, losing power later doesn't reopen the step. The hand-in is a `TalkToNPC` objective completed by the dialogue choice, which takes the shard, completes the quest and gives the reward.

Quest steps:
1. "Reach the datacenter" (`GoToLocation`, the entrance)
2. "Restore the datacenter's power" (`PoweredAt`, the sealed door)
3. "Recover the data shard" (`FetchItem`, not consumed)
4. "Bring the shard to SC-M4X7" (`TalkToNPC`)

### Reward
A standard fusion core, plus one entry from `BLACK_BOX_GUNS`, picked by a generator seeded from the world seed alone:

| Gun | Ammo given |
|---|---|
| Rifle | 12 brass cartridges |
| Burst carbine | 18 brass cartridges |
| Slug thrower | 8 brass slugs |
| Scattergun | 10 scatter shells |

Items go to the inventory, or the ground when it's full.

### Lore
- **SapienCorp's chief executive:** Odessa Crane, who signed the order with the board.
- **The project director:** Dr. Emeric Thale, who ran it.
- **The project:** its name is redacted in the shard ("Project T▒▒▒▒▒▒▒▒").
- **The telemetry:** the reactor was held above its rated output for 41 days. The final entry is Thale's own: "Hold output. The aperture is almost stable."
- **The hand-in:** SC-M4X7 recognises Thale's name, remembers a sealed SapienCorp site "under the river of fire", and asks the player to keep looking. That is the hook for quest 4.

Names are LithicRivers' own. They live in the shard's item description and the dialogue database, and nothing explains the project yet.

## Risks / Trade-offs

- [The datacenter is far for an early player] → It's the same distance class as the facility, and the dormant Datacenter Station links it to the transit network once woken.
- [Power puzzle stalls players without fuel] → The generator room has a sign-like look text ("The generator is cold. Biochar or logs would do."), and logs are everywhere.
- [Cities change the surface hash] → Re-pinned in the same PR, with the reason in the test's comment.
- [Saves from before this change] → `Game::new` queues the datacenter for new worlds; for an old save, the datacenter is queued on load if its structure state is missing, the way old saves get quest chests (`place_quest_chests_for_old_save`).
