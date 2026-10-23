# Design

## Context

- **Feral dogs** are spawned by `spawn_utils::world_spawn_feraldog`, with a seeded inventory rolled from `crafting::FERAL_DOG_LOOT` (`(item, min, max)` ranges). `spawn_utils::spawn_enemy` and `EnemyKind` back the `spawn_enemy` cheat.
- **Dog behavior** comes from `feral_dog_system` (wander, hunt, combat trigger) and `enemy_combat_ai_system` (queued attacks with seeded damage).
- **Death:** `combat_actions::handle_entity_death` turns only feral dogs into corpses (`EntityKind::Corpse` with the dog's inventory).
- **Raw sources** for crafting are the mining drop table plus creature loot (`crafting::raw_sources`).
- **Saving** needs every component registered in `saved_components!`, and a test enforces it.

## Goals / Non-Goals

**Goals:** a second enemy with a distinct pace; a rare, deterministic waste drop; no special cases in death or crafting code.

**Non-Goals:**
- Radiation damage from slimes.
- Slimes splitting when hit.
- Slime spawns around ruined reactors (`add-ruined-reactors` adds those once it lands).

## Decisions

### Loot with chances
- Replace the `(item, min, max)` loot tuples with `CreatureLoot { item, chance_percent, min, max }`. Dogs keep chance 100 and their ranges, so their rolls stay the same.
- **Slime loot:** `[CreatureLoot { item: NuclearWaste, chance_percent: 5, min: 1, max: 1 }]`.
- **Rolls:** at spawn, with a `ChaCha20Rng` seeded from the world seed and spawn position, like dogs. The chance roll happens only for entries with `chance_percent < 100`, so dog rolls don't shift.
- `raw_sources()` includes every loot item with a nonzero chance and max.

### Corpses for any creature with items
`handle_entity_death` leaves a corpse for any dying creature (not the player) whose inventory isn't empty, instead of checking for `FeralDog`. A slime without waste has an empty inventory, so it simply despawns. Dogs behave exactly as before.

### Slime AI
- **Movement:** a `nuclear_slime_system` in the entity AI phase. It notices the player within 6 tiles on the same level and steps one tile toward them every other turn (counted with a per-slime `SlimeAI { turns }` component, saved).
- **Pathing:** *changed during implementation:* a slime doesn't plan a path. It steps straight toward the player, or slides along a wall (one axis), which suits it and avoids the pathfinder aiming at the player's own, blocked tile.
- **Combat** starts through the existing combat trigger: a slime gets `Combat` like a dog.

### Combat numbers (tunable)
- Health: 120 (dog: 80).
- Attack: every 14–18 ticks, versus the dog's 7–10.
- Damage: 10–20 (dog: 5–15).
- Rolls come from `world_state.rng`, as for dogs.

### Spawning
- *Changed during implementation:* not in `ensure_chunk_with_pending_structures`. The renderer generates chunks around the view first, so that hook rarely sees a new chunk.
- Instead, `creature_spawn_system` rolls each underground chunk once, when the player first comes within one chunk of it. The rolled chunks are kept in `Resources::spawn_rolled_chunks`, which is saved.
- **The roll:** `SLIME_CHANCE_PERCENT` (20), seeded by the world seed and the chunk. On success, up to 16 seeded tries find an open, unoccupied tile at least 16 tiles from the player, so no slime pops up in view.
- The surface (`cz >= 0`) and debug worlds never roll. `overhaul-worldgen-depth` moves this onto its feature stage by depth band.
- `Game::new` runs a tick in the bunker at z = -19, so a new game has two slimes underground, far from the start. The golden entity list includes them.

### Loot table
- `CreatureLoot` with `always()` for dogs. `CREATURE_LOOT` lists every creature's table with its plural name, so `raw_sources` and the Item Dictionary's "Carried by nuclear slimes" read from one place (`RawSource::Carried(name)` replaces `RawSource::FeralDog`).

### Combat panel
- Enemies are labeled by kind ("Feral dog", "Nuclear slime") instead of by entity number, which shifted whenever anything new spawned.

### Nuclear waste item
`ItemKind::NuclearWaste` with a sprite. If `add-ruined-reactors` landed first, the item already exists and this change only adds the loot entry.

## Risks / Trade-offs

- [Changing the loot type could shift dog loot] → Chance-100 entries skip the chance roll; the golden test `game_new_matches_golden_entities` and the save-reload identity test catch any shift.
- [5% might feel too rare] → One constant; the drop-rate test pins the 3–7% band so tuning is deliberate.
- [More creatures underground slow ticks] → At most one slime per chunk, rolled once.
