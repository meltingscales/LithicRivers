# Design

## Context

`add-crude-furnace` provides `SmeltingRecipe`, `Furnace` (with optional fuel) and side routing. `add-power-grid` provides `PowerConsumer`, per-mille satisfaction and per-step distribution.

## Goals / Non-Goals

**Goals:** a powered furnace reusing the smelting code.

**Non-Goals:** speed or efficiency tiers, modules, heat.

## Decisions

### Reuse `Furnace` with `fuel: None`
The furnace system branches only on where energy comes from: fuel buffer (crude) or `satisfaction` (electric). Progress per step = `satisfaction` instead of 1000‰. Demand is 90 kW while smelting and 0 otherwise.

### Routing
No fuel slot, so back and both sides feed input.

### Recipe
Electric Furnace: 1 Crude Furnace, 4 IronScrap, 4 ScrapElectronics → 1 (tunable), making the crude furnace a step in its crafting tree.

## Risks / Trade-offs

- [Code paths for two furnace kinds diverge over time] → One system with an energy-source branch; shared tests run against both kinds.
