# Design

## Context

- **Trees** (`plants.rs`): an acorn planted on dirt or grass becomes a sapling with a growth timer, then a tree. Auto-planters plant acorns around themselves, and auto-harvesters chop grown trees (`machines.rs`).
- **Fuel** values live in `power::fuel_energy`. Fluids come with `add-ore-spouts`.
- **The surface** generator is pinned by `the_surface_is_generated_as_before_depth_bands`.

## Goals / Non-Goals

**Goals:**
- Three crops with different jobs.
- Growth that reads at a glance.
- Automation through the existing planter and harvester.
- Deterministic yields.

**Non-Goals:**
- Watering, soil fertility and seasons.
- Animals (`add-animal-farming`).
- Cooking.

## Decisions

### Growth
- Each crop is a tile kind per stage (`WheatSeedling`, `WheatGrowing`, `WheatRipe` and so on) plus a timer entity, as saplings have, so growth survives save and reload.
- Stage times are fixed per crop: wheat 3 game minutes, potatoes 4, carrots 2. By day only once day and night exist.

### Harvesting
- Breaking or auto-harvesting a ripe crop drops its produce (wheat 2, potatoes 3, carrots 2) and 1 to 2 seeds, rolled from `(seed, position, tick of planting)`.
- Breaking an unripe crop returns its seed only.

### Wild crops
- A surface feature step after today's surface generation places a few wild crop tiles on grass in plains and forests.
- This is the first deliberate change to the surface since depth bands, so the surface hash test is re-pinned in this change, with the reason in the test's comment.

### Fermenting
- **Fixed ratios:**
  - 4 wheat makes 100 mB ethanol and 2 straw.
  - 4 potatoes make 160 mB ethanol and 2 mash.
- Straw goes in the fuel table (weak, 300 kJ). Mash and carrots become animal feed in `add-animal-farming`.

## Risks / Trade-offs

- [The surface changes] → Only by adding wild crops on grass tiles, re-pinned on purpose. Scripts that rely on exact surface tiles are reviewed in the snapshot diff.
- [Fluids aren't ready] → Bottled ethanol works until pipes exist.

## Implementation notes

- **Stages:** three tiles per crop (seedling, growing, ripe). A planted crop carries a `CropGrowth` that counts daylight ticks: it grows one factory step at a time on the surface (z >= 0) while the clock isn't at night, turns to its growing tile halfway, and loses the entity when it ripens. Wild crops are ripe tiles with no entity.
- **Yields:** ripe crops are in the mining drop table (`crafting::mining_drop`), so the player and the auto-harvester share one path and the Item Dictionary lists them. Rolls are seeded by world seed and position, like every other mined tile, not by planting tick: a spot always yields the same. Potatoes drop 4-5 potatoes and carrots 3-4 carrots (their produce plus 1 to 2 to replant, since they are their own seeds).
- **Bottled ethanol:** until fluids exist, ethanol is an item: 4 wheat make 5 bottles and 2 straw, 4 potatoes make 8 bottles and 2 mash (the 100 and 160 mB ratios at 20 mB a bottle). A bottle is 1 MJ of fuel in the burner generator, since the gas generator doesn't exist yet; straw is 300 kJ.
- **Fermenter:** a powered furnace (60 kW) working from its own table, with a byproduct slot for straw or mash. It hands out its output first, then its byproduct, through its front and to extractors.
- **Wild crops:** a surface feature step with its own generator (`Feature::WildCrops`) places up to two small patches per chunk on grass in plains and forests. `wild_crops_are_the_only_change_since_depth_bands` maps them back to grass and checks the old surface hash.
- **Showcase:** the wheat farm is the last example; it starts three tiles in so its pole stays out of wire reach of the battery bank's (`examples_keep_their_power_networks_apart`).

