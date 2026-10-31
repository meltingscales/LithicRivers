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

### Spawners (craftable, decided by the user)
- Every spawner spawns one creature within 4 tiles every 30 game seconds while the player is within 24 tiles, up to 4 of its own (tracked by a `SpawnedBy` component, saved).
- **Wild spawners** (`FactoryKind::Spawner` with a fixed creature, placed by worldgen in ruins and dungeons) need nothing. Breaking one, like breaking a placed one, gives the generic spawner item; its creature is not kept.
- **Placed spawners** start empty. Each spawn needs:
  - grid power while it grows (a consumer on the `power-grid` network, like the electric furnace; unpowered, the timer pauses),
  - 1 meat slurry from its slurry slot,
  - a charge from the embryo in its embryo slot. The embryo's kind sets the creature; its grade sets how many spawns it lasts: poor 2, flawed 4, flawless 8. A spent embryo is used up.
- Both slots take items from extractors and belts like a furnace's input, and Interact opens a window like the crude furnace's (slurry, embryo, charges left, power state).
- **Spawner recipe:** iron scrap and scrap electronics, like the electric furnace, so it sits at the same point in the tech tree (exact amounts picked during implementation; the reachability test checks it).
- **Meat slurry:** a hand recipe, 2 meat → 1 meat slurry. A grinder machine can come later.
- **Embryos:** one item per creature kind and grade (`Feral dog embryo (poor)`, ...). Every creature death rolls, seeded like other loot: poor 10%, flawed 3%, flawless 1%, at most one embryo. Creatures from spawners drop them too, which closes the farm loop.

### Fences and gates (decided by the user)
- Fences and gates are factory blocks, so they show in look mode and break back into items. Both are see-through for fog of war and light.
- **Fences** are walls for creatures and the player.
- **Gates** are walls for creatures only: the player's movement treats a gate as walkable, the same way it treats belts (`logistics::walkable`), with no open or closed state.

## Risks / Trade-offs

- [Spawn floods slow the game] → Hard caps per kind and radius, and spawning only every 10 seconds.
- [Spawns feel unfair] → Never in view or close; spawners are visible and breakable.
- [Farms run away: embryo drops outpace use, so a farm never needs tending] → One slurry per spawn bounds output by the meat supply, and the drop chances are tuning numbers to adjust after the playtest.
- [Embryo items multiply with every new creature kind (three each)] → Acceptable while there are two kinds; revisit if the roster grows a lot.
