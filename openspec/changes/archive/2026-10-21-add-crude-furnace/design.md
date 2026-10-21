# Design

## Context

- `add-factory-foundation`: factory blocks with facing, position index, factory steps (5 per game second), chest window.
- `add-conveyor-belts`: `accepts` / `insert` handoff, called with the item and the side it arrives from is derivable from the belt's position.
- `add-power-grid`: fuel values in joules (log 2 MJ, plank 0.5 MJ, stick 0.25 MJ, shavings 0.1 MJ), integer energy accounting, burner generator fuel acceptance.

## Goals / Non-Goals

**Goals:** an unpowered processing machine with fuel efficiency like the burner generator, and a shared smelting table.

**Non-Goals:** ore smelting (no ores yet), multiple output types per input, furnace upgrades, heat or byproducts.

## Decisions

### One output item, per-input yield
"Different fuel outputs for each input" is modeled as different amounts of one `Biochar` item per input (and different times), not separate biochar items. One item keeps inventories, belts and recipes simple; yields carry the difference.

| Input | Consumed | Biochar | Time |
|---|---|---|---|
| Log | 1 | 1 | 4 s |
| WoodenPlank | 2 | 1 | 3 s |
| Stick | 4 | 1 | 3 s |
| Acorn | 4 | 1 | 5 s |
| WoodenShavings | 8 | 1 | 2 s |

Biochar fuel value: 4 MJ. Crude furnace draws 90 kW of fuel energy while smelting (a log smelt costs 0.36 MJ). Numbers tunable.

### Shared `SmeltingRecipe` table and `Furnace` component
`SmeltingRecipe { input, input_count, output, output_count, ticks }` in core, used by every furnace kind. `Furnace { fuel: Option<Storage>, input, output, progress, energy_buffer }`; `fuel` is `None` for furnaces that use power (`add-electric-furnace`).

### Side routing
The handoff interface gets the source direction: `insert(item, from: Facing)`. For a furnace facing F: from behind (opposite of F) → input; from left/right → fuel; from the front → refused. The player's window can put items into any slot.

### Fuel accounting
Same integer joule model as the burner generator: energy is drawn per step only while smelting; a new fuel item is pulled only when the buffer can't cover the step.

### Recipe
Crude Furnace: 12 Stone → 1 (tunable).

## Risks / Trade-offs

- [Biochar makes wood fuel loops net-positive and trivializes power] → The 0.36 MJ smelting cost plus 2 MJ log → 4 MJ biochar is a deliberate density upgrade; tune once the reference farm runs.
- [`insert` signature change ripples into chests/belts] → Chests and belts ignore `from`; one-time mechanical change.

## Implementation Notes

- **Module:** `crates/core/src/furnace.rs`. `Furnace { fuel, input, output: Option<ItemStack>, progress, buffer_j, lit }` holds three single-stack slots of up to 32 items each. Progress counts factory steps, so a 4 s smelt is 20 steps. The planned `fuel: Option<Storage>` for powered furnaces is left to `add-electric-furnace`, which can reuse `SmeltingRecipe` and `Furnace::step` with its own energy source.
- **Fuel:** each step draws 18 kJ (90 kW). The furnace starts a new fuel item only when the buffer can't cover the step. With no fuel it pauses and keeps its progress. With nothing to smelt (or no room for the result), progress resets.
- **Side routing:** `accepts` and `insert` keep their direction-free form for the player's windows. Belts and machine outputs call the new `accepts_from` / `insert_from` with the direction the item is moving. For a furnace facing F, an item moving along F enters its back (input), one moving against F hits its front (refused), and one moving sideways enters a side (fuel). A furnace refuses direction-free inserts; the player uses the furnace window's `furnace::store` / `take`.
- **Lit glyph:** sprites are `crude_furnace_{lit,unlit}_{facing}` (`f` / `F` at 1x). The furnace step swaps the `SpriteRef` when the lit state changes.
- **Window:** a separate furnace window, not the chest's one-list window. The left panel has the Fuel, Input and Output slots, the right panel the player's items. Storing puts the selected item into the selected slot. The title shows facing and idle / smelting % / out of fuel, and the slots panel shows the fuel energy left.
- **Biochar** is fuel at 4 MJ (twice a log) in the shared fuel table, so burner generators take it too.
- **Showcase:** a "Belt-fed crude furnace" example (logs from the back, logs as fuel from the side, biochar out to a chest) sits before the tree farm, which moves 8 tiles east; `look-at-sapling.txt` follows it.
- `ItemStack` now derives `Copy`, `PartialEq` and `Eq`.
