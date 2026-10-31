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
- **Caps:** count creatures of the rule's kind within the radius. At the cap, nothing spawns. A creature only spawns within its rule's radius of the player, so the cap holds around them.
- **Rules:** feral dogs in forest (15% a check, 30% at night, 6 within 64 tiles); nuclear slimes in the shallows (10%) and the deep (20%), 4 within 64 tiles. They replace the old one-time roll per underground chunk, so a new world gets its first slimes on the first check.
- **In view:** core doesn't know the window size, so "the player can see a tile" means: on the player's layer, within 48 tiles (`SIGHT_RADIUS`, wider than a usual window's world view), in line of sight, and lit (surface by day, or within the player's light). Time jumps by whole actions, so the check runs once for each 10-second period crossed (`spawn_check_period`, saved).

### Bases
- No natural spawn on a tile within 8 tiles of a placed light source, or in an area enclosed by fences and walls (a flood fill limited to 64 tiles from the candidate finds no way out). A placed light is any entity with a `LightSource` that isn't the player; no placeable light exists yet, so the rule waits on one (a torch block or lamp). Ground that isn't generated yet counts as a way out of the flood fill.

### Spawners (craftable, decided by the user)
- Every spawner spawns one creature within 4 tiles every 30 game seconds while the player is within 24 tiles, up to 4 of its own (tracked by a `SpawnedBy` component, saved).
- **Wild spawners** (`FactoryKind::Spawner` with a fixed creature, placed by worldgen in ruins and dungeons) need nothing. Structures mark one with the `wild_spawner` tile, which the game turns into a spawner block (like quest chests): dogs on the surface, slimes underground. The user's choice: the ruined outpost has one, and forest chunks that roll a ruin on the surface pick the outpost (buried a layer, cellar below) or the giant corpse half and half, so nothing changes near spawn. Creatures from a spawner carry `SpawnedBy(spawner position)`; a full spawner's timer waits at full and lets one out as soon as one of its own dies. Breaking one, like breaking a placed one, gives the generic spawner item; its creature is not kept.
- **Placed spawners** start empty. Each spawn needs:
  - grid power while it grows (a consumer on the `power-grid` network, like the electric furnace; unpowered, the timer pauses),
  - 1 meat slurry from its slurry slot,
  - a charge from the embryo in its embryo slot. The embryo's kind sets the creature; its grade sets how many spawns it lasts: poor 2, flawed 4, flawless 8. A spent embryo is used up.
- Both slots take items from extractors and belts like a furnace's input, and Interact opens a window like the crude furnace's (slurry, embryo, charges left, power state).
- **Spawner recipe:** 6 iron scrap and 6 scrap electronics, the same point in the tech tree as the electric furnace.
- **Slots and power, as built:** a placed spawner holds up to 16 meat slurry and up to 4 embryos of one kind (the growing one counts). It draws 60 kW while it has slurry and an embryo, and grows as fast as its power allows. Breaking it gives back its slurry and embryos, the growing one whole.
- **Showcase farm:** a chest feeds slurry and dog embryos to the spawner through an extractor and a belt, inside a fenced pen with a gate, with its own generator and pole. Nothing kills the dogs yet, so the embryos they drop only come back to the spawner by hand until machine traps exist.
- **Meat slurry:** a hand recipe, 2 meat → 1 meat slurry. A grinder machine can come later.
- **Embryos:** one item per creature kind and grade (`Feral dog embryo (poor)`, ...). Every creature death rolls, seeded like other loot: poor 10%, flawed 3%, flawless 1%, at most one embryo. Creatures from spawners drop them too, which closes the farm loop. The roll is seeded by where and when the creature died, not where it spawned like its other loot, so creatures a farm keeps spawning on the same tiles don't all drop the same embryo. An embryo the creature drops goes into its corpse.

### Fences and gates (decided by the user)
- Fences and gates are factory blocks, so they show in look mode and break back into items. Both are see-through for fog of war and light.
- **Fences** are walls for creatures and the player.
- **Gates** are walls for creatures only: the player's movement treats a gate as walkable, the same way it treats belts (`logistics::walkable`), with no open or closed state.

## Risks / Trade-offs

- [Spawn floods slow the game] → Hard caps per kind and radius, and spawning only every 10 seconds.
- [Spawns feel unfair] → Never in view or close; spawners are visible and breakable.
- [Farms run away: embryo drops outpace use, so a farm never needs tending] → One slurry per spawn bounds output by the meat supply, and the drop chances are tuning numbers to adjust after the playtest.
- [Embryo items multiply with every new creature kind (three each)] → Acceptable while there are two kinds; revisit if the roster grows a lot.
