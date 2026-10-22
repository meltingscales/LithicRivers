# Design

## Context

- **Quest markers:** `Resources::quest_markers` holds `QuestMarker { name, description, x, y, z, marker_type }`. The Global Map tab lists and draws them.
- **Death (`add-player-death`):** `death::player_died` makes a wreck (a corpse with a `Wreck` marker, the inventory and the old body). `Game::respawn` gives the default body and the starting kit.

## Goals / Non-Goals

**Goals:** never lose a wreck; deleting a marker never deletes items; cheats are plain on/off switches that persist.

**Non-Goals:** waypoints in general (`docs/SPEC-GAPS.md`, `add-waypoints`), and fast travel to a marker.

## Decisions

### Markers
- A death marker is a `QuestMarker` with `QuestMarkerType::Death`, named "Wreck 1", "Wreck 2"... in order of death.
- **Removal:** each tick in the interaction phase (cheap: one check per marker), a death marker whose wreck no longer exists, or is empty, is removed.
- **Deleting by hand** asks first, and only removes the marker: the wreck and its items stay.

### Keep options
- `keep_inventory`: `player_died` leaves the slots on the player. The wreck still spawns, empty, so the marker and the place of death read the same.
- `keep_body`: `respawn` takes the wreck's body, sets every part that isn't Missing to full integrity, and uses that instead of `Body::default()`.
- Both live in `WorldSettings`, so they save with the world and show in the world's settings. The cheats toggle them and say the new state.

## Risks / Trade-offs

- [A cheat used by accident] → Each toggle logs its new state, and they need the world's "Allow cheats" setting.
