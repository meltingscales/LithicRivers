# Proposal

## Why

The crude furnace needs fuel delivered and burns some of the value it makes. Once the power grid exists, an electric furnace lets players smelt with power alone: one less belt to run, and it pairs naturally with generators burning biochar.

## What Changes

- **Electric furnace**: smelts using the same smelting table as the crude furnace, but has no fuel slot and draws power from the grid instead, only while smelting. It runs proportionally slower when the grid is short of power and stops when unpowered.
- **Two slots**: input (from the back or either side) and output (pushed out the front).
- **Furnace window** shows input, output, progress and the network's power satisfaction.
- **Recipe** for the electric furnace.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `smelting`: adds the electric furnace (power instead of fuel, no fuel slot, side routing without fuel).

## Impact

- Core: `FactoryKind::ElectricFurnace` using `Furnace { fuel: None, .. }` plus `PowerConsumer`.
- Client: glyph and window variant without a fuel slot.
- Depends on `add-crude-furnace` (smelting table, `Furnace` component) and `add-power-grid`.
