# Proposal

## Why

Automation needs energy, and a Factorio-style grid gives the factory a shape: generators somewhere, poles carrying power across the map, machines drawing from it. Starting with burner generators makes the tree farm self-sustaining (logs in, power out) and gives wood a second use.

## What Changes

- **Burner generator**: burns fuel items (logs, planks, sticks, wood shavings) to produce power, up to a fixed maximum. It only burns as much fuel as the grid actually uses: an idle grid burns nothing, and a partly used fuel item keeps its remaining energy.
- **Power poles**: each pole powers every generator and machine within a generous supply radius, and links to other poles within a longer wire reach, forming networks. Poles across chunk edges link normally.
- **Networks**: all generators and consumers connected through poles share power. When demand exceeds supply, every consumer gets the same fraction and works proportionally slower.
- **Pole lines on screen**: wires between linked poles are drawn over the terrain and blink on and off every second, so the terrain behind them stays visible.
- **Inspecting**: interacting with a generator or pole shows the network's supply, demand and the generator's fuel.
- **Recipes** for the generator and the pole.

## Capabilities

### New Capabilities
- `power-grid`: Generators, poles, networks, power distribution to consumers, fuel use, and how power lines are drawn.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `Generator { fuel: Storage, energy_buffer }`, `PowerPole`, `PowerConsumer { demand }` components; network building (connected components of pole links); generator and power-distribution factory systems (run before machines each step).
- Client: generator and pole glyphs, wire rendering with 1-second blink (from the app clock, so headless runs are deterministic), network info window.
- Depends on `add-factory-foundation` and `add-crafting-tree`. Consumers arrive in `add-tree-automation`; `add-view-modes` adds a power-only view.
