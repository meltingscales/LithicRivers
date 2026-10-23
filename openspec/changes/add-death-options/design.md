# Design

## Context

- **Quest markers:** `Resources::quest_markers` holds `QuestMarker { name, description, x, y, z, marker_type }`. The Global Map tab lists and draws them.
- **Death (`add-player-death`):** `death::player_died` makes a wreck (a corpse with a `Wreck` marker, the inventory and the old body). `Game::respawn` gives the default body and the starting kit.

## Goals / Non-Goals

**Goals:** never lose a wreck; deleting a marker never deletes items; cheats are plain on/off switches that persist.

**Non-Goals:** waypoints in general (`docs/SPEC-GAPS.md`, `add-waypoints`), and fast travel to a marker.

## Decisions

### Markers
- A death marker is a `QuestMarker` with `QuestMarkerType::Death`, named "Wreck 1", "Wreck 2"... in order of death (`PlayerState.deaths`, saved).
- **Description:** "Cause: <cause>. Holds Nail x10, Log x3, Torch x3, 5 more." — the three biggest stacks, so the player knows what is worth going back for. The Global Map shows the selected marker's description under it.
- **World view:** the wreck's sprite already marks the spot, so there is no extra blinking `X`. The Global Map draws death markers as `X`.
- **Removal:** after each tick's systems (`death::prune_death_markers`, which returns at once without death markers), a death marker with no non-empty wreck at its position is removed.
- **Deleting by hand:** on the Global Map, `global_map.DELETE_MARKER` (`x`) on a selected death marker opens an "Are you sure?" prompt; `global_map.CONFIRM_DELETE` (`y`) deletes and `global_map.CANCEL_DELETE` (`n`, Escape) keeps it. The prompt takes every key until answered. Only the marker goes: the wreck and its items stay. Other markers can't be deleted.
- **Marker order:** the markers list is sorted by distance; the selection, the arrow and deleting all use that same order (`markers_by_distance`).

### Keep options
- `keep_inventory`: `player_died` leaves the slots on the player. The wreck still spawns, empty, so the marker and the place of death read the same.
- `keep_body`: `respawn` takes the wreck's body, sets every part that isn't Missing to full integrity, and uses that instead of `Body::default()`.
- Both live in `WorldSettings`, so they save with the world and show in the world's settings. The cheats toggle them and say the new state.

## Risks / Trade-offs

- [A cheat used by accident] → Each toggle logs its new state, and they need the world's "Allow cheats" setting.
