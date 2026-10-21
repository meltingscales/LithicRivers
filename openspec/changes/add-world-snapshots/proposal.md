# Proposal

## Why

A world can be lost to one bad decision: a fight gone wrong, a mined-out support, a factory torn down by mistake. Saving over the only save makes that permanent. Players also come back to a world after a break and can't remember what they were doing. Automatic snapshots fix both:
- **Rollback:** the player can roll back to any recent point.
- **Branching:** rolling back keeps the abandoned future as its own branch, like a git history.
- **Action log:** each snapshot records what happened since the one before it.

## What Changes

- **Automatic snapshots** of the current world, taken every 5 game minutes of play and whenever the world is saved, stored inside the world's folder.
- **A branching timeline:**
  - Each snapshot records its parent.
  - Rolling back to an older snapshot first snapshots the present, then continues from the chosen point, so new snapshots start a new branch and the old future stays reachable.
- **An action log per snapshot:**
  - What happened since its parent: the messages logged in that span, with ticks.
  - A one-line summary: game time, position, health, and what changed in the inventory.
- **A Timeline screen** (from the Menu tab):
  - The snapshots as a tree, newest first, with branches indented and the current position marked.
  - Selecting a snapshot shows its summary and action log, with a confirmed **Roll back here**.
- **Pruning:** only the 10 most recent snapshots are kept, plus the one the world currently descends from, so disk use stays bounded. Pruned parents show as "(pruned)" in the tree.

## Capabilities

### New Capabilities
- `world-snapshots`: When snapshots are taken, what they hold, the branching timeline, rolling back, the action log, and pruning.

### Modified Capabilities
<!-- None. Snapshots live inside world folders (world-saves) but don't change how worlds are saved or loaded. -->

## Impact

- **Client:**
  - A snapshot store in each world folder (`snapshots/<id>.json` plus `snapshots/index.json`), written with the existing version 5 save format (`App::save_to_string`).
  - The automatic snapshot trigger checks game time after each action.
  - The Timeline screen and the rollback flow.
  - A Menu-tab key for the Timeline (`ui.OPEN_TIMELINE`).
- **Core:** nothing new; snapshots reuse the save format, and the message log already records ticks.
- **Config:** the unused `SNAPSHOTS_FOLDER` setting is removed in favor of per-world folders.
- **Headless:** snapshots go to the run's temporary saves folder, and timestamps come from the fake clock, so output stays deterministic.
