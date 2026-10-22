# Proposal

## Why

When the robot dies (its power source or torso is destroyed), the game marks it Dead, logs "You have died!", and carries on: there is a `TODO: Trigger game over state` in `combat_actions.rs`. A dead player can still be looked at but not meaningfully played, and nothing tells them what happened or what to do. For the Steam MVP, death needs a clear outcome. `docs/TODO.md` suggests a "backup body" system, which fits the robot protagonist: a robot can be rebuilt.

## What Changes

- **A system-failure screen** when the robot dies: what destroyed it, where, and two choices.
  - **Boot backup body:** the robot wakes in a fresh, barely working backup body at its backup point.
  - **Quit to title:** the world is saved as it is (the dead body included), so the player can come back to the same choice.
- **The old body stays where it fell** as a wreck holding everything the robot carried, like a creature's corpse. Walking back to it and looting it recovers the items, so death costs time and risk, not the whole inventory.
- **The backup body:**
  - Starts like a new game's body: the left arm missing, one leg damaged, the rest working. Upgrades and repairs made since are lost with the old body.
  - Carries the starting kit only: everything else stays with the wreck.
- **The backup point:** the bunker spawn at first. Later changes can add ways to move it (a repair station, for example); this change only stores it.
- **No penalty beyond that:** the world, factories and quests keep running and keep their state.

## Capabilities

### New Capabilities
- `player-death`: What happens when the robot dies: the system-failure screen, the wreck, booting a backup body, and the backup point.

### Modified Capabilities
<!-- None: combat keeps its rules; this handles the moment after the player's death. -->

## Impact

- **Core:**
  - `handle_entity_death` for the player spawns a wreck (a corpse with the player's inventory and a `Wreck` marker) and keeps the player entity, marked Dead.
  - `Game::boot_backup_body` moves the player to the backup point, gives it a fresh starting body and kit, and removes the Dead marker.
  - `PlayerState::backup_point` (saved, defaulting to the bunker spawn).
- **Client:**
  - A system-failure modal that blocks other input while the player is dead, with its two choices (keys and click targets).
  - The wreck's sprite.
- **Tests:** dying spawns the wreck with the inventory, booting resets the body and position, quitting and reloading shows the screen again, and nothing is lost or duplicated.
