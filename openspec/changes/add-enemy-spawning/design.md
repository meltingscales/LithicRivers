# Design

## Context

- `creature_spawn_system` rolls each underground chunk once, when the player first comes within a chunk of it, with a seeded generator per chunk; rolled chunks are saved.
- Creatures: feral dogs (chase within 15 tiles) and nuclear slimes. Both block movement and are fought through `Combat`.
- Determinism: everything from the seed, the tick and positions.

## Goals / Non-Goals

**Goals:**
- Areas refill slowly.
- Never more than the caps.
- Never spawning in view or inside bases.
- Farms that work.
- Deterministic.

**Non-Goals:**
- New creature kinds (`add-ranged-combat` adds the military android).
- Machine traps that kill creatures.
- Breeding.

## Decisions

### Rules and timing
- **When:** every 10 game seconds (1000 ticks), each rule is checked for the chunks around the player that are out of view. A chunk that spawned waits a cooldown (5 game minutes) before spawning again.
- **Rolls:** seeded by `(seed, chunk, tick / 1000, rule)`, so the same play gives the same spawns.
- **Caps:** count creatures of the rule's kind within the radius. At the cap, nothing spawns.

### Bases
- No natural spawn on a tile within 8 tiles of a placed light source, or in an area enclosed by fences and walls (a flood fill limited to 64 tiles from the candidate finds no way out).

### Spawners
- `FactoryKind::Spawner { creature }` spawns one creature within 4 tiles every 30 game seconds while the player is within 24 tiles, up to 4 of its own (tracked by a `SpawnedBy` component, saved).
- Breaking it gives a spawner core item for that creature.

### Fences and gates (decided by the user)
- Fences and gates are factory blocks, so they show in look mode and break back into items. Both are see-through for fog of war and light.
- **Fences** are walls for creatures and the player.
- **Gates** are walls for creatures only: the player's movement treats a gate as walkable, the same way it treats belts (`logistics::walkable`), with no open or closed state.

## Risks / Trade-offs

- [Spawn floods slow the game] → Hard caps per kind and radius, and spawning only every 10 seconds.
- [Spawns feel unfair] → Never in view or close; spawners are visible and breakable.
