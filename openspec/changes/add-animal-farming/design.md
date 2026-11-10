# Design

## Context

- **Sheep:** `stumbling_sheep_system` wanders them with the seeded world RNG, and they have health but no drops.
- **Machines:** powered blocks with inventories and progress (`machines.rs`); power demand per machine.
- **Fences:** come with `add-enemy-spawning` (block creatures, not sight).

## Goals / Non-Goals

**Goals:**
- Farms that run themselves.
- A reason to care about meat.
- Graded embryos with tunable odds.
- Deterministic.

**Non-Goals:**
- More animal kinds (cows, chickens).
- Taming or riding.
- Animal genetics.

## Decisions

### Breeding
- A sheep eats one feed item (from the player or a feeder) and becomes `Fed` for 2 game minutes.
- Two fed sheep within 3 tiles breed: a lamb spawns on an open tile between them, chosen by seeded roll, and both enter a 5-minute cooldown.
- Lambs grow up in 5 minutes.
- **Cap:** no breeding when 16 or more sheep are within 8 tiles, so farms can't flood the game.

### Feeders and counters
- **Feeder:** every 20 game seconds it feeds the nearest unfed adult within 3 tiles.
- **Counter:** counts by `EntityKind` within its square each factory step. Its output is a SapienNet value (`add-sapiennet`) once that exists; until then it only shows the count.

### The livestock processor
- **While powered:** 10 damage every 2 game seconds to every non-player creature on the 3 tiles in front of it.
- **Drops:** creatures killed this way drop into its inventory (sheep gain a drop table: 1–2 meat, 0–1 leather).

### Bio-vats
- **Batch:** 4 meat + 2 mash + 1 scrap electronics, taking 2 game minutes at full power. Brownouts slow it.
- **Grade roll:** seeded by `(seed, vat position, batch count)`.

  | Grade | Base chance |
  |---|---|
  | Deformed | 70% |
  | Wrinkly | 25% |
  | Perfect | 5% |

  A batch with an extra 2 meat shifts 10 points from deformed to wrinkly and perfect evenly.

## Risks / Trade-offs

- [Culling feels grim] → It is grim, and the lore leans into it: SapienCorp built these. The description says what it is.
- [Sheep floods] → The breeding cap.

## Implementation notes

- **Module:** `livestock.rs` holds feeding, breeding, the four blocks and their factory step, which runs after machines. `Fed`, `BreedCooldown` and `Lamb` are saved components on sheep.
- **Feeding:** the player feeds an adjacent hungry sheep through Interact ("Feed sheep a carrot"), which uses the first carrot or mash they carry. Fed, resting and young sheep refuse.
- **Lambs:** a lamb is a sheep with `Lamb`, 25 health and its own sprite, and it doesn't breed or eat. The lamb's tile is one of the open tiles beside either parent, chosen by a roll seeded by the first parent's tile and the tick.
- **Processor:** strikes the 3 tiles across its front (ahead, and either side of ahead), every 2 game seconds of full power. It wants power only with an animal there. It never targets the player or NPCs: it only acts on sheep, feral dogs, nuclear slimes and mining bots. Its kills leave the usual corpse, which it empties into its inventory and clears. They make no messages and don't count as the player's kills (`handle_machine_kill`).
- **Sheep drops:** sheep now leave a corpse like other creatures. Their drops (1-2 meat and 0-1 leather, or 1 meat for a lamb) are rolled when they die, seeded by position and tick, because sheep carry no inventory while alive.
- **Counter:** Interact steps its square from 5x5 up to 15x15 and back. The look panel shows its count; the SapienNet output waits for `add-sapiennet`.
- **Bio-vat:** inputs and embryos share its inventory. Belts and the player can only put ingredients in, and extractors only take embryos out. It stops at 8 embryos.
- **Embryo uses:** the Standard Torso now needs a wrinkly embryo and the Advanced Torso a perfect one. A deformed embryo renders into 2 meat slurry for spawners. The reachability check now counts machine products (furnace, fermenter and bio-vat outputs) as obtainable.
- **Showcase:** a fenced sheep pen with a feeder, a counter, a processor in its south fence and a bio-vat outside.

