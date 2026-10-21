# Proposal

## Why

With planting, growth, belts and power in place, the last piece of the first automation loop is machines that do the farm work: one that plants acorns and one that chops grown trees and sends the wood down a belt. Together they turn a patch of grass into a self-running wood supply that can also fuel its own generators.

## What Changes

- **Auto-planter**: a powered machine that takes acorns (from a belt feeding it, or placed in it by the player) and plants them on free grass/dirt in a square around it.
- **Auto-harvester**: a powered machine that chops grown trees in a square around it and outputs the logs and acorns in its facing direction: onto a belt, into a chest, or into an internal buffer if nothing is there.
- Both work at a speed proportional to the power they get, and stop when unpowered or when their output is full.
- Both work across chunk edges like other factory blocks.
- **Recipes** for both.
- **A reference farm** headless script: planter, harvester, belt, chest, generator and poles, running unattended and filling the chest.

## Capabilities

### New Capabilities
- `tree-automation`: Auto-planter and auto-harvester behavior, inputs, outputs, work area and power use.

### Modified Capabilities
<!-- None -->

## Impact

- Core: `FactoryKind::AutoPlanter` / `AutoHarvester`, a `Machine { input, output, progress }` component, machine factory systems (run after power, before belts), handoff interface implemented for machine inputs.
- Client: machine glyphs with facing, inspect window (buffers, progress, power).
- Depends on `add-factory-foundation`, `add-crafting-tree`, `add-tree-farming`, `add-conveyor-belts` and `add-power-grid`.
