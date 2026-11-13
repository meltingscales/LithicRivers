# Proposal

## Why

Quest 3 ends with SC-M4X7 remembering "a sealed site under the river of fire," and nothing answers it yet. A fourth quest should follow that thread into the deep, reveal what Thale was building (the aperture) without explaining it, and unlock the next tier of power. The planned `add-ruined-reactors` change already describes that tier (ruined reactors, uranium and a thermoelectric generator), so this change folds it in and gives it a story reason to exist.

## What Changes

- **Ruined reactors** (from `add-ruined-reactors`, adjusted):
  - Rare, partly buried multiblock ruins on the surface, three layouts, placed from the world seed and chunk.
  - Built from reactor casing (slow to mine, drops reactor casing scrap), nuclear waste blocks (drop nuclear waste) and a few uranium deposits (drop uranium chunks).
  - Mining a tile now takes as long as its hardness says: reactor casing takes five times as long as rock; everything else is unchanged.
- **Thermoelectric generator** (from `add-ruined-reactors`, adjusted):
  - A generator that holds one heat source (nuclear waste or a uranium chunk) and supplies constant power forever without using it up: 30 kW with waste, 60 kW with uranium.
  - Crafted from iron scrap, scrap electronics and reactor casing scrap. The old plan used iron ingots from `add-ore-spouts`, which isn't built, so this change uses iron scrap; ingots can replace it when ore spouts land.
- **Thale's Annex:** a guaranteed structure, one per world, under a lava river in the deep. A SapienCorp access shaft on the surface leads down to it.
  - Its blast door is a sealed door (from `add-ruined-city-quest`) on a network with no generator, and the annex's outer reactor room holds casing and nuclear waste. To open it, the player builds a thermoelectric generator from what the annex holds and places it on the door's network.
  - A dormant "Annex Station" links it to the transit network.
- **Quest 4, "Under the River of Fire":** offered by SC-M4X7 once "Black Box" is complete.
  1. Reach Thale's Annex.
  2. Power the annex door.
  3. Recover Thale's journal from his lab.
  4. Bring the journal to SC-M4X7.
- **Reward:** an advanced arm and a uranium chunk.
- **Lore:** the journal shows the aperture was a gate Thale built to reach "the far side," powered by the overloaded reactor. It opened once, for four seconds, and something on the other side noticed. It also hints the maintenance frames (the player's kind) were built to remember the aperture's shape. It doesn't say what the player is for. It points toward where the aperture's parts were made, as the hook for quest 5.
- **Removed:** the `add-ruined-reactors` change folder, now covered here.

## Capabilities

### New Capabilities
- `ruined-reactors`: where ruined reactors appear, what they're made of, what each part drops, and tile hardness.
- `thermoelectric-power`: the thermoelectric generator, its heat sources and the power it gives.
- `sealed-site-quest`: Thale's Annex, its door puzzle, the quest's steps, the reward and the reveal.

### Modified Capabilities
None. Quest 3's capability (`black-box-quest`) is still in an unarchived change; this quest only reads its completion.

## Impact

- **Core:**
  - Tiles `ReactorCasing`, `NuclearWasteDeposit`, `UraniumDeposit` and `TreasureQuest3`, with mining drops; a hardness on `MiningDrop` that scales the mining action's cost.
  - A separate seeded roll for reactor ruins, so other structures don't shift.
  - `FactoryKind::Thermoelectric` with a one-slot heat source, registered as a generator.
  - `QuestType::SealedSite`, its objectives, dialogue, hand-in and reward; the annex placed like the datacenter, with its marker and station name.
  - New items: uranium chunk, reactor casing scrap, thermoelectric generator, Thale's journal.
- **Content:** three reactor ruin plans and the annex (shaft and annex) as `plan.toml`, checked with `just util-structure-preview`.
- **Client:** the generator window (heat source slot), sprites, `tp_quest annex`.
- **Depends on:** the sealed door and quest 3 (`add-ruined-city-quest`), the power grid, transit stations, lava rivers (all merged).
