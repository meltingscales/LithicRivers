# Proposal

## Why

The game has no ending. The user's idea (IDEAS-INBOX.md) sets one, and it isn't a rocket ship: a strange **biomechanical teleporter**, an 11-by-11 multiblock that needs a continuous drip of endgame items to run.
- **A test of the factory:** finishing means building a factory that keeps producing those items for a long time, a sustained-production goal like Factorio's.
- **Planning tools:** an item-flow view shows which inputs are keeping up.
- **A secret:** the teleporter stays unknown until the player has done many quests, under the slow-reveal principle in AGENTS.md and `docs/QUESTS.md`.

This is not `add-teleporters`, which adds small teleporters for travel. This is the game's goal.

## What Changes

- **The Biomechanical Teleporter:** an 11-by-11 multiblock of frame blocks around a core.
  - Each frame block is crafted from endgame materials: petrogen fuel, enriched uranium, perfect embryos, strange matter, advanced cells.
  - Once assembled, it needs a set of input items at fixed rates per game minute, through input ports on its edges fed by belts and pipes.
- **The drip:**
  - Each input has a required rate.
  - The teleporter charges while every input keeps up, for a long stretch (target: about 2 hours of game time at full rates).
  - Any input falling short slows the charge, and a long shortage drains it.
  - A full charge opens the teleporter: the ending.
- **The item-flow window:** every input with its required rate, its actual rate over the last minute and the last ten minutes, and a color:
  - **green:** comfortably above the requirement
  - **yellow:** about at it
  - **red:** below and falling

  It also shows the charge and an estimate of the time left.
- **Production over time:** the factory-wide view in `add-factory-stats` (item flow per minute, by item and source) is the planning tool. This change depends on it and links to it from the teleporter's window.
- **Hidden until earned:**
  - Its recipes, blocks and lore are locked until a late quest reveals them.
  - Before that, the Item Dictionary and crafting tab don't list them, and nothing hints at the goal.
- **Large factories:** factory blocks already run everywhere (`docs/TECH-TREE.md`, "Simulated everywhere"), so no chunk loading is needed. A performance test runs a factory of a target size (for example 2,000 blocks) to keep factory steps within the tick budget.

## Capabilities

### New Capabilities
- `endgame-teleporter`: The teleporter multiblock, its input rates and charge, the item-flow window, its quest-gated reveal, and the large-factory performance check.

### Modified Capabilities
<!-- None in openspec/specs: crafting visibility gates are new, and factory-stats is still a change. -->

## Impact

- **Core:**
  - Multiblock detection for the 11-by-11 frame.
  - Input ports with rate tracking (a rolling count per input per game minute).
  - The charge model.
  - A "revealed" flag set by a quest, saved; recipes and dictionary entries filtered by it.
  - A factory performance benchmark test.
- **Client:**
  - the item-flow window with its colors and time estimate
  - the teleporter's sprites and an 11-by-11 frame art
  - the ending, at least a closing screen, for a later change to make into a sequence
- **Depends on:**
  - `add-factory-stats` (flow data)
  - `add-advanced-processing`, `add-animal-farming` and `add-power-cells` (input materials)
  - the quest line that reveals it (a quest change to come)
