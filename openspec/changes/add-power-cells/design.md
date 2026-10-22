# Design

## Context

- **Power grid** (`power.rs`): generators with energy buffers, poles linked into networks, consumers with demand, and per-step supply against demand with a satisfaction fraction.
- **Inventory stacks** are `ItemStack { kind, qty }`, with no per-item data.

## Goals / Non-Goals

**Goals:**
- Storage that makes grids forgiving.
- Portable power for gear.
- Charge visible everywhere it matters.
- Deterministic.

**Non-Goals:**
- Wireless charging.
- Cells as fuel for generators.
- Power-armor suits.

## Decisions

### Batteries
- **Each grid step:**
  1. Generators supply.
  2. Consumers demand.
  3. Any surplus charges batteries, up to their rate, in position order.
  4. Any deficit discharges them, up to their rate.
- A battery's stored joules are saved on the block.

### Cell charge in stacks
- `ItemKind::PowerCell(tier)` stacks don't stack: a cell's `qty` is always 1.
- A new `ItemStack` field `charge_j: Option<u64>`, serde default `None`, holds its charge. Only cells use it, so every other stack is unchanged in saves.

### Fusion-core charging
- **Per game second:**

  | Core state | Charge |
  |---|---|
  | Damaged | 20 J |
  | Functional | 50 J |
  | Enhanced | 100 J |

- It goes into the first carried cell that isn't full, in inventory order. A Missing core can't happen while the robot lives.

### Gear
- `use_power(game, joules) -> bool` draws from the fullest carried cell and returns false if no cell has enough. The biome compass uses it.

## Risks / Trade-offs

- [Adding a field to `ItemStack` touches saves] → Serde default `None`. The save-reload identity test and old saves loading cover it.
- [Batteries hide fuel problems] → The pole info box shows stored energy draining, and `add-factory-stats` graphs it.
