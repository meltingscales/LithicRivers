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
