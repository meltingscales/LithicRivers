# Proposal

## Why

Wood is the only fuel, and it's bulky and weak. Smelting wood into biochar gives the tree farm a processing step, a denser fuel for generators and furnaces, and the first machine that doesn't need power, so players can start processing before they build a grid.

## What Changes

- **Crude furnace**: an unpowered factory block with three slots: **fuel**, **input** and **output**. It burns fuel to smelt the input, and only burns while it is smelting; leftover energy from a partly burned fuel item is kept.
- **Biochar**: a new fuel item, denser than wood. Each wood-type input gives a different amount of biochar:
  - log → 1 biochar
  - 2 wooden planks → 1 biochar
  - 4 sticks → 1 biochar
  - 4 acorns → 1 biochar
  - 8 wood shavings → 1 biochar

  Each input also takes a different time to smelt.
- **Biochar as fuel** for crude furnaces and burner generators (higher energy than a log).
- **Belts and facing**: items entering from the back go to input, from either side go to fuel, and finished biochar is pushed out the front, so a furnace slots into a belt line.
- **Recipe** for the crude furnace (stone-based).

## Capabilities

### New Capabilities
- `smelting`: The smelting table (what smelts into what, how long, and how much energy), furnace slots and how belts feed them, and biochar as fuel.

### Modified Capabilities
<!-- None: biochar's fuel value is added to the fuel table owned by add-power-grid's power-grid spec; if that change has been archived first, this change adds a MODIFIED delta for the burner-generator fuel list. -->

## Impact

- Core: `ItemKind::Biochar`, a shared `SmeltingRecipe` table, `FactoryKind::CrudeFurnace` with `Furnace { fuel, input, output, progress, energy_buffer }`, a furnace factory system; handoff rules by side.
- Client: furnace glyph with a lit/unlit state, a three-slot furnace window (reusing the chest window layout).
- Depends on `add-factory-foundation` and `add-crafting-tree`; works with `add-conveyor-belts` and shares fuel values with `add-power-grid`. `add-electric-furnace` reuses the smelting table.
