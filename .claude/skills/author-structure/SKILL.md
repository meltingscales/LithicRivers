---
name: author-structure
description: Design, write, check and preview LithicRivers multiblock structures (dungeons, ruins, outposts, bunkers, ships, temples, quest locations) as plan.toml files under crates/client/assets/structures/. Use when asked to make, sketch, generate, edit or fix a structure, building, dungeon, room layout or quest location, or when a structure lint test fails. docs/STRUCTURES.md is the full reference.
---

# Author a structure

Structures are `<name>.lrstructure` folders in `crates/client/assets/structures/`. Read `docs/STRUCTURES.md` first: it has the format, the plan shapes, placement rules and the special tile keys (enemy spawns, loot, scrap, the quest locker, the player spawn).

## Workflow

1. **Pin down the brief:** purpose, rough size, how many levels and how they connect, and what it holds (enemies, loot, quest tiles). Ask only if something material is missing; otherwise pick sensible defaults and say what you picked.
2. **Write a plan:** create `crates/client/assets/structures/<name>.lrstructure/plan.toml`. Start from `ruined_outpost.lrstructure/plan.toml` (two levels, rooms, rubble, stairs) or `small_temple.lrstructure/plan.toml` (one room). Rules of thumb:
   - Layer 0 is the bottom. A buried structure (bunker, facility) puts its top layer at the placement z, so its entrance is usually the top layer.
   - Use `room` for walled spaces, `corridor` to join them, `stairs` for a column on every layer it spans, `fill` for solid rock or rubble, `put` for single blocks, spawns and loot.
   - Leave the outside as the `background` (`.` keeps the world's own tiles).
   - Every character you use must be in `[legend]`, with a real tile key.
3. **Build:** `just util-structure-build`. It writes `shape_layers.txt` and the `data.json` legend, and only touches files that change.
4. **Preview as text:** `just util-structure-preview <name>`. Read every layer back and fix the plan until it reads right and ends with "No problems found."
5. **See it in the renderer:** a scratchpad script for `just client-blind`:
   ```
   seed 12345
   size 100x30
   world structure:<name>
   key ui.MENU_PREV
   screen layer-0
   key action.LOOK_TOGGLE
   key movement.MOVE_UP
   screen layer-1
   ```
   `MOVE_UP` alone only climbs stairs; look mode is what moves the view between layers.
6. **Check:** `cargo test -p lithicrivers-core structure` runs the lint (sizes, legend, stairs alignment, plan matches layers) over every structure.
7. **Place it, only if asked:** queue a `PendingStructure` from code (as `Game::new` does for the facility) or add it to `structure::structures_list()` for random placement. A new spawn tile kind also needs `structure::spawn_entity_for_block`.

## Rules

- Edit the plan, never a planned structure's `shape_layers.txt`: the lint fails when they disagree. Structures without a plan (the bunker, the facility) are hand-written; edit their `shape_layers.txt` directly.
- Follow `docs/ART-DIRECTION.md` for names, lore and any new tiles: content is LithicRivers' own.
- Report the preview (or the layers that matter) back to the user so they can judge the layout without running anything.
