# Proposal

## Why

Power exists only on the grid: generators feed poles, and poles feed machines next to them. Anything the robot carries can't use it, and the grid can't store a surplus. The user's idea (IDEAS-INBOX.md): power cells in three tiers that store power, charged at battery blocks on the grid or slowly by the robot's own fusion core, to power gear such as the biome compass (`add-biome-compass`).

## What Changes

- **Battery blocks:** a grid block that stores energy.
  - It charges from a network's surplus and discharges into it when demand exceeds supply, so a grid with batteries rides out a fuel gap.
  - Its window shows its charge, and charging and discharging rates.
- **Power cells, three tiers:**

  | Tier | Capacity | Made from |
  |---|---|---|
  | Crude cell | small | early materials (iron scrap, biochar paste) |
  | Standard cell | medium | mid-game materials, scrap electronics |
  | Advanced cell | large | endgame materials (from `add-advanced-processing`) |

- **Charging:** a cell placed into a battery block's cell slot charges from the battery, and from the grid through it.
- **The fusion core charges the cell the robot carries:** slowly, at a rate set by the core's state and tier (a damaged core charges slower, an Enhanced one faster). One cell at a time, the first one that isn't full.
- **Gear that runs on cells:** a gear item that needs power draws it from the fullest carried cell when used, and says "no charge" when every cell is empty. The biome compass is the first; later gear (lamps, tools) uses the same rule.
- **Grid reports:** the power pole's info box shows the network's stored energy and battery charge rates.

## Capabilities

### New Capabilities
- `power-cells`: Battery blocks on the grid, power cells in three tiers, charging from batteries and the fusion core, and gear drawing from cells.

### Modified Capabilities
- `power-grid`: networks include battery storage in supply and demand.

## Impact

- **Core:**
  - `FactoryKind::Battery` with stored energy (saved), and the grid step charging and discharging it.
  - Power cell items with a per-item charge. Charge is item data, so cells in stacks need a stack-of-one rule or a charge per stack; decided in design.
  - Fusion-core trickle charging.
  - A `use_power(joules)` helper for gear.
- **Client:**
  - the battery window
  - cell charge shown in the inventory and the Item Dictionary
  - the pole info box's storage line
- **Showcase:** a generator, a battery bank and a machine, where the machine keeps running briefly after the fuel runs out.
- **Depends on:** the power grid. The advanced cell needs `add-advanced-processing` materials.
