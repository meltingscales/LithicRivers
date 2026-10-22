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
