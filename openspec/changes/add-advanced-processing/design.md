# Design

## Context

- **Machines** (`machines.rs`): powered factory blocks with inventories, a `Machine` progress state and `PowerConsumer` demand. Factory steps run on elapsed game time, and every block is simulated wherever it is.
- **Fluids:** `add-ore-spouts` introduces fluid kinds, millibucket amounts, pumps, pipe networks and machine fluid inputs.
- **Fuel:** `power::fuel_energy` maps items to joules.

## Goals / Non-Goals

**Goals:**
- Chains with several steps.
- A use for every byproduct.
- Rare finds that stay rare and deterministic.
- Every machine works with belts and pipes.

**Non-Goals:**
- The nuclear reactor itself (`add-ruined-reactors`).
- Heat simulation beyond each machine's own progress.

## Decisions

### The distillation column
- A multiblock of three tiles in a row; the first is the controller.
- Slurry in at the bottom. Each step splits a fixed batch into tar, oil and gas by fixed ratios (tunable constants), output at its tier's tile.
- An incomplete column shows "incomplete" in its window and does nothing.

### Uranium
- **Fixed ratios per step:** for example, 4 crushed crust to 1 yellowcake.
- **The centrifuge's rare result:** rolled from `feature_rng`-style seeding by `(seed, machine position, batch count)`, so the same factory gives the same enrichment over time. About 1 enriched uranium per 20 batches; the rest is depleted.

### Washing
- Each wash: 1 residue + 100 mB water.
- **Loot table:**

  | Result | Chance |
  |---|---|
  | Grit | 80% |
  | Iron scrap | 15% |
  | Scrap electronics | 4.9% |
  | Strange matter | 0.1% |

  Rolled per batch with the same seeding as the centrifuge.
- **Rates** come from tunable constants. The showcase examples run each chain and a test checks every output appears within a fixed number of batches (strange matter excepted, tested by a seeded long run).

### Byproducts
- Light gas burns in a gas generator (a generator taking a fluid fuel).
- Grit makes gravel fill.
- Depleted uranium is kept for future armor.

## Risks / Trade-offs

- [Too many machines at once] → Each chain can ship on its own: washing first (simplest), then petrogen, then uranium.
- [Rare drops feel bad] → The rates are visible in the Item Dictionary, and byproducts always come out.
