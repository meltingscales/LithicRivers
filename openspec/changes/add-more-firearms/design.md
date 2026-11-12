# Design

## Context

- **Guns today** (`crates/core/src/firearms.rs`): a `Gun` table (ammo item, range, accuracy at 1 tile, falloff, damage, ticks, casing). `loaded_guns` lists equipped guns with rounds, and `execute_shoot` fires each through `systems::ranged::fire` (range, line of sight, seeded roll, shot line and sound).
- **Slimes** (`nuclear_slime_system`): notice within 6 tiles, step every other turn, melee when adjacent; loot rolled at spawn into the inventory, left in a corpse.
- **Embryos and spawners** (`spawner.rs`): `embryo(creature, grade)` maps creatures to embryo items; machines have none.
- **Natural spawns** (`creature_spawn.rs`): `SpawnRule`s with a `SpawnPlace` (a depth band), chance, cap and radius, checked every 10 game seconds.
- **Lava** fills the deep band's rivers (`add-molten-rivers`). Wild spawners and transit stations show how worldgen marks positions in `GameWorld` lists for the game to turn into entities.
- **Power cells** keep their charge as item data (`power_cells::charge`), and the tele-access device already pays from the fullest carried cell.

## Goals / Non-Goals

**Goals:**
- Each gun plays differently, and each tier has a reason to switch.
- The new traits are data on the gun table, resolved by one Shoot path.
- Sulfur becomes steady enough for gunpowder-hungry guns, by building near vents or farming slimes.

**Non-Goals:**
- Splash that damages the robot or the world (no terrain destruction or friendly fire).
- Gun mods, reloading, ammo swapping in one gun.
- Fluids. Sulfur vents grow tiles and don't wait for `add-ore-spouts`' pumps and pipes; if that lands, a vent could later also feed it.

## Decisions

### Gun traits are fields on `Gun`
`Gun` gains `pellets` (per round, default 1), `burst` (rounds per Shoot, default 1), `stun_percent`, `pierce`, `splash` and `ammo: Ammo`, where `Ammo` is `Item(ItemKind)` or `Charge(joules)`. `execute_shoot` loops bursts, then pellets, and each roll goes through `ranged::fire`. Alternative: a trait object per gun. Rejected: every gun is the same resolution with different numbers, and the table stays easy to read and tune.

- **Bursts** use one round each and stop early when rounds run out. `loaded_guns` counts a burst gun as loaded with at least one round.
- **Charge** comes from the fullest carried cell that holds a whole shot's charge, the same pick as the tele-access device. A gun with too little charge isn't loaded.
- **Piercing** walks the shot's line out to the gun's range and rolls against each living enemy on it, stopping at the first tile that blocks sight. The shot line is drawn to the farthest tile reached.
- **Splash** applies only on a hit: every other enemy within 1 tile of the target takes the splash damage. The robot is never in the list.
- **Stun** uses the existing `Stunned` component and duration that Tackle uses.

### The guns

| Gun | Tier | Ammo | Range | Hit at 1 / lost per tile | Damage | Ticks | Traits |
|---|---|---|---|---|---|---|---|
| Nail driver | early | 1 nail | 4 | 65% / 10% | 5–9 | 8 | none |
| Scattergun | early | 1 scatter shell | 3 | 75% / 20% | 4–7 per pellet | 12 | 5 pellets |
| Burst carbine | mid | brass cartridges | 8 | 75% / 6% | 10–16 | 12 | burst of 3, casings |
| Slug thrower | mid | 1 brass slug | 5 | 60% / 10% | 24–34 | 16 | 30% stun, casing |
| Arc caster | late | 1 MJ of charge | 8 | 85% / 5% | 18–28 | 10 | charge |
| Riverglass lance | late | 4 MJ of charge | 14 | 95% / 2% | 30–40 | 20 | pierce, charge |
| Isotope launcher | late | 1 isotope canister | 7 | 80% / 6% | 30–45, splash 15–25 | 18 | splash |

The numbers are a starting point for the playtest. A crude cell holds 2 arc shots, a standard cell 20.

### Ammo and gun recipes

| Makes | From |
|---|---|
| 2 scatter shells | 1 paper + 1 gunpowder + 3 iron shot |
| 2 brass slugs | 2 shell casings + 2 gunpowder + 1 iron scrap |
| 1 isotope canister | 1 nuclear waste + 1 brass ingot + 1 gunpowder |
| Nail driver | 2 iron scrap + 1 wooden plank + 2 string |
| Scattergun | 1 pipe gun + 2 iron scrap + 1 nail |
| Burst carbine | 3 brass ingots + 2 iron scrap + 2 scrap electronics + 1 plasteel scrap |
| Slug thrower | 2 brass ingots + 4 iron scrap + 1 plasteel scrap |
| Arc caster | 6 scrap electronics + 3 plasteel scrap + 1 riverglass + 1 crude power cell |
| Riverglass lance | 6 riverglass + 4 plasteel scrap + 4 scrap electronics + 1 standard power cell |
| Isotope launcher | 4 brass ingots + 3 plasteel scrap + 2 nuclear waste + 2 scrap electronics |

The late guns lean on riverglass, plasteel and nuclear waste, which only the deep provides.

### Sulfur slime
A new `EnemyKind::SulfurSlime` that reuses the nuclear slime's movement (one system for both slimes, keyed by a shared slime marker). It has 100 health (feral dog 80, nuclear slime 120), attacks for 8–16 every 12–16 ticks, and its loot is always 1 to 3 sulfur. It gets three embryo grades like the other creatures, so `roll_embryo` and the spawner already handle it.

Natural spawning gains a `SpawnPlace::NearLava` (deep band, within 4 tiles of lava), with a rule like the deep nuclear slime's (cap 4). Alternative: put them in the deep band anywhere. Rejected: tying them to the rivers makes them a place you go for sulfur, beside the vents.

### Sulfur vents
- **Tiles are the state.** A vent is a `SulfurVent` tile and its crystals are `SulfurCrystal` tiles; there is no vent entity and nothing per vent to save. Alternative: worldgen marks spots for the game to turn into entities, as wild spawners do. Rejected: chunk generation can't write to `GameWorld` lists (`generate_chunk` takes `&self`), and the cap can be counted from the tiles.
- **Placement:** in the deep band, about one chunk in six (`VENT_CHUNK_ODDS`) tries up to 32 random open-floor tiles with lava within 4 tiles inside the chunk, and puts a vent on the first, with the chunk's own feature generator so nothing else shifts.
- **Unbreakable:** the vent tile has no mining drop. Mining it logs "The sulfur vent can't be broken."
- **Growth:** once per 30-game-second period crossed (time jumps by whole actions, as with natural spawns; `Resources::vent_growth_period` remembers the last), every vent within 24 tiles of the player on their layer grows one `SulfurCrystal` on an open, walkable neighbor with nobody on it, picked from the world seed, the vent's position and the period. The cap is 4 crystals within 1 tile.
- **Crystals** mine like a deposit: 1 to 2 sulfur, leaving air. They block movement, so a vent ringed with crystals needs mining to reach. Mining bots drill them too.

### Combat panel
The odds line under the selected enemy reads each gun's chance per pellet or round, adds "x5" for pellets and "x3" for bursts, and shows charge guns' shots left from the fullest cell ("Arc Caster 80% (2 shots)").

## Risks / Trade-offs

- [The isotope launcher trivializes groups] → Canisters cost nuclear waste, which is rare, and splash only reaches tiles beside the target.
- [The lance pierces past the target] → It still stops at walls, and it is the slowest gun.
- [Vents make sulfur too easy] → The 4-crystal cap and 30-second growth keep it to about 2 crystals (3 sulfur) a game minute per vent, and only while the player is nearby.
- [More enemy kinds to keep in sync] → The slime shares the nuclear slime's AI and the generic loot and embryo paths. `EnemyKind::ALL` drives the cheat suggestions and spawn tests.
- [New tiles shift the surface hash] → New `TileKind` variants go at the end of `define_tiles!`, as `AndroidSpawn` did.

## Migration Plan

Existing saves load unchanged: equipped guns and items keep their kinds, and vents appear only in chunks generated after the update. Already-generated deep chunks have no vents, which is acceptable for a pre-release game.
