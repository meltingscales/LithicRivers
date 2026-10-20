# Design

## Context

`add-factory-foundation` provides factory blocks, the position index, factory steps (`FACTORY_STEP_TICKS = 20`, 5 per game second) and chunk-safe access. `App::clock` (`Clock`) drives time-based rendering; it is fake and deterministic in headless runs.

## Goals / Non-Goals

**Goals:** a simple, deterministic grid with fuel efficiency and proportional sharing.

**Non-Goals:** accumulators/batteries, other generator types (steam, solar), wire colors or manual wire placement, transformers, power loss over distance.

## Decisions

### Units
Energy in joules (J), power in watts (W = J per game second). Per factory step a consumer needs `demand_w / 5` J.

| Item | Fuel value |
|---|---|
| Log | 2 MJ |
| WoodenPlank | 0.5 MJ |
| Stick | 0.25 MJ |
| WoodenShavings | 0.1 MJ |

Burner generator max output: 90 kW. All numbers tunable.

### Networks
Rebuilt only when a pole, generator or consumer is placed, removed or loaded (dirty flag), not every step: union-find over poles linked within reach 8 (Chebyshev distance), then each generator/consumer joins the network of every pole whose supply square (radius 3) covers it; a block covered by poles of two networks merges them. Each network gets a stable id (lowest pole position) so iteration order is deterministic.

### Per-step distribution
For each network: `demand = Σ consumer demand of working consumers`; `available = Σ generator min(max_output, remaining fuel energy + next fuel item)` per step; `satisfaction = min(1, available / demand)`; each generator supplies `demand * satisfaction * (its available / total available)` and deducts exactly that from its energy buffer, pulling the next fuel item only when the buffer can't cover it. Idle (demand 0) means nothing is deducted. Consumers read `satisfaction` and scale their progress.

### Wires and blink
Wires are a client-side render of pole links: Bresenham line between pole positions, drawn with `─│╱╲` style characters (or `*` where ambiguous) over terrain but under entities. Visibility: `(app.clock.animation_millis() / 1000) % 2 == 0`. Drawn in the normal view and in `add-view-modes`' power view.

### Recipes
| Recipe | Ingredients | Result |
|---|---|---|
| Burner Generator | 8 Stone, 4 IronScrap, 2 ScrapElectronics | 1 |
| Power Pole | 1 Log, 1 IronScrap | 2 |

## Risks / Trade-offs

- [Floating-point drift breaks determinism] → Integer joules (u64) and fixed-point satisfaction (per-mille); no floats in the simulation.
- [Wire lines clutter small screens] → Blink keeps terrain visible; view modes can hide them.
- [Network rebuild cost] → Only on changes, not per step.
