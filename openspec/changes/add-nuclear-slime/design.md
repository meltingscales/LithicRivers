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
- **Movement:** a `slime_system` in the entity AI phase. It notices the player within 6 tiles on the same level and steps one tile toward them every other turn (counted with a per-slime `SlimeAI { turns }` component, saved).
- **Pathing:** it reuses the dog's pathfinding step.
- **Combat** starts through the existing combat trigger: a slime gets `Combat` like a dog.

### Combat numbers (tunable)
- Health: 120 (dog: 80).
- Attack: every 14–18 ticks, versus the dog's 7–10.
- Damage: 10–20 (dog: 5–15).
- Rolls come from `world_state.rng`, as for dogs.

### Spawning
- In `Game::ensure_chunk_with_pending_structures`, a chunk generated for the first time with `cz < 0` rolls `SLIME_CHANCE` (seeded by chunk coordinates).
- On success, it places one slime on a passable tile chosen by the same seeded generator.
- The surface (`cz >= 0`) never rolls.

### Nuclear waste item
`ItemKind::NuclearWaste` with a sprite. If `add-ruined-reactors` landed first, the item already exists and this change only adds the loot entry.

## Risks / Trade-offs

- [Changing the loot type could shift dog loot] → Chance-100 entries skip the chance roll; the golden test `game_new_matches_golden_entities` and the save-reload identity test catch any shift.
- [5% might feel too rare] → One constant; the drop-rate test pins the 3–7% band so tuning is deliberate.
- [More creatures underground slow ticks] → At most one slime per chunk, rolled once.
