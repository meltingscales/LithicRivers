# Proposal

## Why

The Global Map only lists markers the game places: quests, the bunker, the factory and wrecks. Players have no way to mark a base, an ore field or a vat they want to find again. The user asked for waypoints the player can create, update and delete on the Global Map, while quest markers stay put.

## What Changes

- **Waypoints:** a new marker type the player owns, listed with the other markers by distance and drawn on the map with its own symbol.
- **Create:** a key on the Global Map adds a waypoint at the player's position, named "Waypoint N" (the lowest free number), and selects it.
- **Update:** with a waypoint selected, one key renames it (typing, Enter keeps, Escape cancels) and another moves it to the player's position.
- **Delete:** the existing delete key and yes/no prompt work on waypoints as well as wrecks.
- **Quest markers can't be deleted:** the delete key on any other marker says so and changes nothing.
- Waypoints are saved with the world, like every marker.

Out of scope: placing a waypoint anywhere other than the player's position (clicking the map), colors or icons per waypoint, and showing waypoints in the world view.

## Capabilities

### New Capabilities
- `map-waypoints`: player-made waypoints on the Global Map: creating, renaming, moving and deleting them, and which markers can't be deleted.

### Modified Capabilities
<!-- None: death-options' wreck-marker deletion is unchanged. -->

## Impact

- **Core:** `QuestMarkerType::Waypoint`, which markers are deletable, and naming new waypoints (`resources.rs`).
- **Client:** Global Map input (`app/input/global_map.rs`) and panel (`ui/panels/global_map.rs`), new `global_map` keybinds in `default_config.rs`, and `App::typing()` while renaming.
- **Saves:** markers are already saved; the new variant needs nothing else.
