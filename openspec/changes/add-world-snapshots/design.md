# Design

## Context

- **Worlds** are folders under `SAVES_FOLDER` with `world.json` (the version 5 save, including the client section) and `meta.json` (`crates/client/src/worlds.rs`). `App::save_world` writes both. `App::save_to_string` / `load_from_str` save and load in memory.
- **Save completeness:** version 5 saves restore everything, and `save_reload_anywhere_changes_nothing` proves it.
- **Game time:** 100 ticks = 1 game second (`factory::TICKS_PER_SECOND`).
- **The message log** keeps the last 200 messages as `"[tick] text"`.
- **Config:** `SNAPSHOTS_FOLDER` exists and is unused.

## Goals / Non-Goals

**Goals:** cheap, automatic, branching snapshots with readable history; bounded disk use; deterministic in headless runs.

**Non-Goals:**
- Diff or incremental snapshots (each one is a full save).
- Merging branches.
- Manual named snapshots (could come later).
- Snapshots of unsaved `skip-splash` worlds.

## Decisions

### Storage
`<world>/snapshots/index.json` holds `[{ id, parent, tick, created_unix, summary, log: [String] }]` plus `head`, the id the world currently descends from. Each snapshot's data is `<world>/snapshots/<id>.json`. Ids count up per world and are never reused. `head` is also written to `meta.json`, so the world list can show it.

### Triggers
- **Timer:** after every action, `App` checks whether `tick - last_snapshot_tick >= 30_000` (5 game minutes).
- **Save:** `save_world` also takes a snapshot.
- Both go through `App::take_snapshot`, which serializes once with `save_to_string`.

### Action log
`take_snapshot` copies the message log entries whose tick is after the parent's tick (at most 200). The summary compares the player's position, health and inventory with the values stored on the parent's index entry, as counts, not the full save.

### Rollback
`roll_back(id)`:
1. `take_snapshot` (the present becomes a leaf).
2. Load `<id>.json` into the app.
3. Set `head = id`.
4. Save the world, so `world.json` matches.

The next snapshot gets `parent = id`, which starts a new branch.

### Pruning
After each `take_snapshot`, keep the set formed by the 10 highest ids plus `head`, and delete the other files and index entries. A pruned parent is shown as "(pruned)" in the tree.

### Timeline screen
- A new full-screen panel, opened by `ui.OPEN_TIMELINE` (default `T`) from the Menu tab.
- The tree is built from parent links and drawn with indentation per branch.
- Navigation uses the `title.*` keys. Selecting a snapshot shows its details; "Roll back here" opens a confirmation.

## Risks / Trade-offs

- [Full saves are large (tiles)] → 11 snapshots per world at most; a test checks the count and that deleted files are gone.
- [Snapshot while a modal is open] → Snapshots only capture saved state, the same as a save, so transient UI isn't captured.
- [Message log only keeps 200 entries] → A snapshot's log may start mid-span; the summary still covers the whole span.
