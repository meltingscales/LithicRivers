# Proposal

## Why

When the robot dies (its power source or torso is destroyed), the game marks it Dead, logs "You have died!", and carries on: there is a `TODO: Trigger game over state` in `combat_actions.rs`. A dead player can still be looked at but not meaningfully played, and nothing tells them what happened or what to do. For the Steam MVP, death needs a clear outcome. `docs/TODO.md` suggests a "backup body" system, which fits the robot protagonist: a robot can be rebuilt.

## What Changes

*Scope decided by the user: a simple respawn for now, because the backup body isn't designed yet.*

- **A system-failure screen** when the robot dies, saying what destroyed it, with two choices.
  - **Respawn at the bunker:** the robot comes back at the bunker's spawn point with the default body and the starting kit. This is a stand-in until the backup body is designed.
  - **Quit to title:** saves the world as it is, dead body included, so the player comes back to the same screen.
- **The old body stays where it fell** as a wreck that keeps:
  - **Everything the robot carried,** looted like a creature's corpse.
  - **Its body parts:** the wreck stores the old body, so a later change can let the player salvage parts from it.

  Walking back and looting the wreck recovers the items, so death costs time and risk, not the whole inventory.
- **The respawn point:** where a new game put the player (the bunker spawn), stored when the world is created.
- **No penalty beyond that:** the world, factories and quests keep running and keep their state.

## Capabilities

### New Capabilities
- `player-death`: What happens when the robot dies: the system-failure screen, the wreck, booting a backup body, and the backup point.

### Modified Capabilities
<!-- None: combat keeps its rules; this handles the moment after the player's death. -->

## Impact

- **Core:**
  - The player's death, from any cause, spawns a wreck: a corpse with the player's inventory and old `Body`, and a `Wreck` marker. The player entity stays, marked Dead.
  - `Game::respawn` moves the player to the respawn point, gives it the default body and the starting kit, and removes the Dead marker.
  - `PlayerState::respawn_point` (saved; older saves fall back to the configured default position).
- **Client:**
  - A system-failure modal that blocks other input while the player is dead, with its two choices (keys and click targets).
  - The wreck's sprite.
- **Tests:** dying spawns the wreck with the inventory, booting resets the body and position, quitting and reloading shows the screen again, and nothing is lost or duplicated.
