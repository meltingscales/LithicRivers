# Design

## Context

- Enemies follow one pattern: a marker component, an `EntityKind`, an `EnemyKind` (cheat and spawn rules), a spawn function in `spawn_utils`, an AI system in `entity_ai`, and the shared combat trigger and enemy combat AI. The nuclear slime is the closest template: no pathfinding, a step straight toward the player.
- Natural spawns go through `SPAWN_RULES` in `creature_spawn.rs` (never in view, never within 16 tiles, caps per radius, each chunk at most once).
- The starting bunker is placed at (10, 10, 1) and reaches down to z = -19, deep inside the deep band. Its walls are plain rock.
- `crafting::mine_tile` mines a tile with seeded drops. The player and machines both use it.

## Goals / Non-Goals

**Goals:** a deep-only enemy that walls don't stop; loot that rewards fighting it; no unfair start.

**Non-Goals:**
- Pathfinding around obstacles it can't drill (it slides along them as slimes do).
- Bots drilling between layers.
- Bots that keep mining when no player is near.
- Bot spawners or bot embryos.

## Decisions

### Chase: like a slime, every turn, drilling what blocks it
The bot takes one step a turn toward the player: diagonal first, then along each axis. If none of those tiles is open and the first one is drillable, it drills that one instead. Drillable tiles are natural rock and the four deposits, decided by a single `drillable(tile)` function. Drilling goes through `crafting::mine_tile`, so drops are the same seeded drops the player gets, and they go into the bot's inventory.
Alternative: A* through rock with a dig cost. Rejected: more code than the behavior needs, and a straight-line driller reads clearly to the player ("it's coming through the wall").

### Hearing, not sight
The bot notices within 10 tiles on its layer (Chebyshev distance), ignoring line of sight. That is what makes walls unsafe. Dogs see 15 to 22 tiles and slimes 6.

### Combat
Health 150 (slime 120, dog 80). Drill hits do 12 to 22 damage every 10 to 13 ticks: harder than a dog, faster than a slime. Rolled from the world RNG like the others.

### Death leaves a wreck
Bots count as creatures in `handle_entity_death`, so their inventory becomes a corpse entity drawn with the existing `wreck` sprite. The log says "The mining bot grinds to a halt." Since `MINING_BOT_LOOT` always gives at least one scrap electronics, there is always a wreck.

### Embryos are optional per kind
`spawner::embryo(kind, grade)` returns `Option<ItemKind>`, and `MiningBot` maps to `None`. `roll_embryo`, `embryo_kind` and the Item Dictionary's loot list skip `None`.

### Keeping away from the bunker
`SpawnRule` gets `away_from_bunker: i64`, the minimum horizontal distance from `game::BUNKER_POS` (a new constant that replaces the literals in `Game::new`). Dogs and slimes use 0. Bots use 128, two chunks.
