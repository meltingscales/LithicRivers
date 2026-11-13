# Design

## Context

- **Quests 2 to 4** set the pattern: a guaranteed structure queued as a `PendingStructure` (`Game::new`, and `queue_datacenter` for old saves, which now also queues the annex), ordered objectives checked each tick (`GoToLocation`, `FetchItem`, `PoweredAt`), dialogue nodes on the repaired android gated by `requires_quest_complete`, quest chests that carry their item (`TreasureQuest1` to `TreasureQuest3`), and shared `complete_all` and `reward` helpers in `quests.rs`.
- **Lava crust:** placing a plain block (stone, plank block, stone brick) on lava leaves walkable lava crust.
- **Renewable chains today:**
  - Tree farms make logs, which furnaces turn into biochar.
  - The fermenter makes ethanol and mash from wheat or potatoes.
  - The sheep farm and livestock processor make meat; meat makes meat slurry.
  - The bio-vat makes graded biomech embryos from meat, mash and scrap electronics, so a perfect one comes about one batch in several.
  - Everything mined (uranium, riverglass, plasteel, scrap electronics) is hand-mined; nothing mines it automatically.
- **Factory stats** (`factory_stats`) already keep per-minute item flow. The Production tab reads it.
- **`add-endgame-teleporter`** is the old plan: an 11-by-11 multiblock with rate tracking, a charge model and a reveal gate, fed by unbuilt materials.

## Goals / Non-Goals

**Goals:**
- An ending reachable with today's game: a sustained-production test built only on chains that can be automated.
- The secret kept until quest 5.
- A clear, ordered last quest that tests a mechanic the earlier ones didn't (bridging lava).

**Non-Goals:**
- The far side itself (worldgen, crafting, quests or a boss past the Aperture): a later change.
- Automatic mining, petrogen, enriched uranium and strange matter (their own changes; they can become extra inputs).
- An animated cinematic beyond a scripted closing sequence of screens.

## Decisions

### The foundry
`sapiencorp-foundry` is a surface plan (not buried) of one layer, about 30 by 20, at `FOUNDRY_POS` = (900, -460, 0), queued with the other quest structures. Its layout:
- **Entrance yard:** scrap heaps and two android spawns (mining bots stay in the deep band, so none guard the foundry).
- **Casting hall:** a molten casting pit, a band of lava five tiles wide across the hall, with stone heaps on the near side to make the bridge from (the player can also bring their own blocks).
- **Pattern vault**, beyond the pit: a `TreasureQuest4` chest holding the aperture casting patterns, and some rare loot.
- **No transit station:** the walk out is part of the last journey.

The "cross the casting pit" objective is a `GoToLocation` on the vault side, reachable only over the pit.

### The reveal gate
`Resources::revealed: BTreeSet<Reveal>` (saved, default empty) with `Reveal::Aperture`. Items have a `revealed_by(item) -> Option<Reveal>`; the Crafting tab, the Item Dictionary, the hotbar picker and placement skip items whose reveal isn't set. Finishing "The Foundry" inserts `Reveal::Aperture`. The cheat `reveal aperture` sets it for testing. Alternative: unlock recipes by owning the patterns. Rejected: dropping the patterns would hide the goal again.

### The multiblock
- **Blocks:** `ApertureCore`, `ApertureFrame` and `AperturePort`.
- **Layout:** the core at the center of an 11-by-11 square, the frame filling its border, and ports at five fixed border positions (centers of the north, east, south and west edges, and the north-west corner), each taking one input in a fixed order.
- **Completeness:** checked from the core whenever an Aperture block is placed or broken. The core keeps an `Aperture { complete, missing: Option<Position>, charge_milli, history, opened }` component, saved.

### Inputs and rates

| Input | Port | Required per game minute |
|---|---|---|
| Perfect biomech embryo | north-west corner | 0.2 (2 per 10 minutes) |
| Meat slurry | north | 4 |
| Ethanol | east | 6 |
| Mash | south | 3 |
| Biochar | west | 8 |

Each port counts arrivals into the core's per-minute history (10 buckets, like `factory_stats`), and takes from belts and extractors like a machine input. These rates need a farm of roughly a dozen bio-vats, a sheep farm, a wheat field with fermenters and a tree farm with furnaces: a big factory, but buildable. They are constants, tuned in the playtest.

### Charge
Per game minute, with `fraction` = each input's arrivals over its required rate across the kept minutes (up to ten), capped at 1, so slow inputs like embryos aren't short in every minute one doesn't happen to arrive:

- every input at 1 or more: +1000 milli-points
- otherwise: + 1000 × the lowest fraction
- any input below 0.5 for three minutes in a row: −1000 instead

The charge is kept between 0 and 120,000. At 120,000 milli-points, `opened` is set. Integer milli-points keep it deterministic.

### The flow window
Interact on any Aperture block opens it. It shows:
- each input with its required rate, the last minute's and the ten-minute rate, and its color (green at 110% or more and not falling, yellow from 95% to 110%, red below 95% or falling over the last three minutes);
- the charge, and the time left at the ten-minute rates;
- while incomplete, the first missing position instead.

### The ending
When `opened` becomes true:
1. The client plays a closing sequence: a few full-screen text pages over the Aperture's art, written in the slow-reveal voice and showing nothing of the far side. Any key steps through the pages.
2. The credits follow.
3. The game returns to the world, saved, with the Aperture's sprite open.

Interact on the open Aperture logs "The far side is not ready for you yet." That is the hook for the far-side change.

### Recipes (a one-time cost)

| Makes | From |
|---|---|
| Aperture frame (x4) | 4 plasteel scrap, 2 riverglass, 1 reactor casing scrap |
| Aperture port | 1 frame, 4 scrap electronics, 1 advanced arm |
| Aperture core | 1 advanced fusion core, 6 riverglass, 4 uranium chunks, the casting patterns |

The android hands the patterns back on hand-in, and the core uses them up, so the reveal flag (not the patterns) keeps the recipes visible.

### The bench
A `scripts/bench/aperture-farm.txt` scenario in the factory showcase world with the Aperture built (cheated) and a fed showcase factory around it, timing a minute of play.

## Risks / Trade-offs

- [The rates feel like a chore, or too easy] → They're constants; the playtest tunes them, and the flow window shows exactly what is short.
- [Perfect embryos are random] → The rate is low (2 per 10 minutes) and averaged over ten minutes; a short run of bad grades only dips the charge.
- [Bio-vats need scrap electronics, which is hand-mined] → One per batch, so a few hundred over two hours; a later automatic-mining change removes the last hand step.
- [Spoiling the secret] → Tests check the Crafting tab, the Item Dictionary and the picker before and after the reveal, and the casting patterns' description says only what the foundry was casting.
- [The closing sequence fights the headless snapshots] → It reads time from `app.clock` and steps on keys, so scripts can drive it.

## Migration Plan

New worlds get the foundry with the other quest structures; old saves get it queued on load. `Resources::revealed` defaults to empty. `add-endgame-teleporter` is removed in this change's PR.
