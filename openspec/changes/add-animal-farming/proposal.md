# Proposal

## Why

Sheep wander the surface but do nothing, and meat and leather only come from feral dogs. The user's idea (IDEAS-INBOX.md): animal farms that are factories, with:
- **auto-feeders**
- **animal counters** that report how many animals are nearby
- **a lore-friendly way to cull them**
- **bio-vats** that give the meat a purpose, by growing biomechanical embryos for crafting

## What Changes

- **Breeding:** two sheep that are fed carrots (`add-crops`) within a few tiles of each other produce a lamb, which grows into a sheep. Fed animals can't breed again for a cooldown.
- **Auto-feeder:** a block that holds feed (carrots, or mash from fermenting) and feeds animals within 3 tiles, one at a time, on a timer. An empty feeder does nothing.
- **Animal counter:** a block that counts animals of each kind within an adjustable square (5x5 up to 15x15). It shows the count and sends it as a SapienNet signal (`add-sapiennet`), so a farm can stop feeding when it's full.
- **Culling, lore-friendly:** a **SapienCorp livestock processor**, a salvaged slaughter unit from the old world's agri-plants. It damages non-player creatures in front of it while powered, and their drops go into its inventory.
  - It never targets the player or tame companions.
  - Its description doesn't hide what it is.
- **Bio-vats:** a powered block that turns meat, mash and scrap electronics into **biomechanical embryos** over time.
  - Each embryo grows to one of three grades, rolled per batch: **deformed** (common), **wrinkly** (uncommon) or **perfect** (rare).
  - The vat's feed quality (more meat per batch, steady power) improves the odds.
  - Embryos are crafting ingredients: perfect embryos for advanced backup-body parts (`add-backup-bodies`), wrinkly and deformed ones for lesser parts and biomechanical machines.

## Capabilities

### New Capabilities
- `animal-farming`: Breeding with feed, auto-feeders, animal counters, the livestock processor and bio-vats with graded embryos.

### Modified Capabilities
<!-- None: sheep keep their wandering behavior, and gain feeding and breeding. -->

## Impact

- **Core:**
  - A `Fed { until }` component and breeding cooldowns, plus lambs growing up.
  - New factory blocks:
    - an auto-feeder
    - an animal counter (with a SapienNet output once that exists)
    - the livestock processor (a powered damage area)
    - a bio-vat (powered, with seeded grade rolls)
  - New items: feed, embryos.
- **Client:** windows, sprites, and the counter's number in look mode.
- **Showcase:** a fenced sheep farm with a feeder, a counter, a processor and a bio-vat.
- **Depends on:**
  - `add-crops` (carrots and mash)
  - `add-enemy-spawning`'s fences, for pens
  - the power grid
  - `add-sapiennet`, for the counter's signal (the counter works without it)
