# Design

## Context

- `add-factory-foundation`: factory blocks, facing, position index, 5 factory steps per game second, `Game::factory_tile`.
- `add-tree-farming`: `Sapling` tiles with `Growth`; grown trees are plain `Tree` tiles; chopping logic in `mining_system` (seeded drops).
- `add-conveyor-belts`: `accepts` / `insert` handoff interface; belts run after machines.
- `add-power-grid`: `PowerConsumer { demand }`, per-step network satisfaction (per-mille).

## Goals / Non-Goals

**Goals:** two machines that close the wood loop, deterministic and power-aware.

**Non-Goals:** filters, configurable areas, upgrades/modules, other crops, inserter arms.

## Decisions

### Shared `Machine` component
`Machine { input: Storage, output: Storage, progress_permille: u32, action_cost: u32 }`. Each step, if the machine has work, it sets `PowerConsumer.demand` (planter 30 kW, harvester 60 kW) and adds `satisfaction` to progress; when progress reaches `action_cost` (2 or 3 game seconds in steps × 1000), it acts and resets. No work → demand 0.

### Fixed tile order
Work tiles are scanned row by row from the north-west corner of the radius-2 square, skipping the machine's own tile, so the same farm behaves the same every run.

### Chopping reuses tree logic
Extract the tree-chop drop logic from `mining_system` into a function both the player and the harvester call, so drops stay identical (same seeded RNG by position).

### Output push
After acting, and every step, a machine pushes one item per step from `output` into the faced tile through the handoff interface (belt, chest, machine input).

### Recipes
| Recipe | Ingredients | Result |
|---|---|---|
| Auto-Planter | 6 WoodenPlank, 4 Nail, 2 ScrapElectronics | 1 |
| Auto-Harvester | 4 Stone, 6 IronScrap, 3 ScrapElectronics | 1 |

### Reference farm
`scripts/agent/farm.txt`: spawn-free setup using `cheat giveitem` for machines, fuel and acorns, places the layout, passes 30 game minutes, dumps `state`. Also added as a `cargo test` that checks logs produced > logs burned, and as a snapshot.

## Risks / Trade-offs

- [Balance: the farm can't sustain its generator] → The reference-farm test pins the property; tune numbers until it holds.
- [Harvester and player both chopping the same tree in one tick] → Player actions run before factory steps; the harvester re-checks the tile before acting.
