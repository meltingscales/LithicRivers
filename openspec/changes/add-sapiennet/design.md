# Design

## Context

- **Power networks** (`power.rs`): poles linked by wires, rebuilt into networks when dirty, with supply and demand per network each step.
- **Factory steps** run in a fixed order on elapsed game time, and every block is simulated wherever it is.

## Goals / Non-Goals

**Goals:**
- Readable logic that a player can debug.
- Deterministic signal evaluation.
- Nothing silently does anything: a block with an unmet condition says so.

**Non-Goals:**
- Wireless signals.
- Per-tick timing tricks (signals update once per factory step).
- Scripting (`add-sapiennet-scripting`).

## Decisions

### Networks
- Cables are a factory block kind. A network is a connected set of cable tiles (4-neighbor), plus every factory block touching one, found by flood fill when cables change (`dirty` like power).
- A block touching two networks reads the sum of both.

### Evaluation order, each factory step
1. Sensors write their values into their network's signal map.
2. Logic blocks evaluate in position order, each reading the previous step's outputs. This avoids loops within one step, as Factorio's combinators do.
3. Blocks with conditions read the totals before acting.

### Expressions
- **Grammar:** signal names, integers, `+ - * / %`, comparisons (`< <= > >= == !=`), `AND`, `OR`, `NOT` and parentheses.
- **Parsing:** done once when set, into a saved expression tree. A parse error shows in the editor and the block keeps its last valid condition.
- **Integer math:** division by zero gives 0, so a bad formula never crashes the factory.

### Unmet conditions
A block whose condition is false shows "Waiting: sheep < 12 is false (sheep = 14)" in its window and in look mode.

## Risks / Trade-offs

- [Too complex for most players] → Conditions are optional, and the showcase teaches the two most useful patterns.
- [Performance with many networks] → Networks rebuild only when cables change, and evaluation is a pass over sensors and logic blocks.
