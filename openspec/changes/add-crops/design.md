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
