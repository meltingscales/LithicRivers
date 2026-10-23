# Proposal

## Why

`add-player-death` leaves the robot's wreck where it died, holding everything it carried. In a big world that wreck is easy to lose. Testing and creative play also want death to cost nothing: keep the inventory, keep the body. The user asked for both in IDEAS-INBOX.md.

## What Changes

- **Death markers:**
  - Dying adds a marker at the wreck on the Global Map, with a brief description: what destroyed the body and the biggest stacks it holds. In the world view the wreck's own sprite marks the spot.
  - Looting the wreck empty removes its marker.
  - A marker can also be deleted by hand from the Global Map, after an "Are you sure?" yes/no prompt ("Delete the marker for Wreck 1 at (12, -40, -3)? Its items stay there.").
  - Several deaths keep several markers, numbered.
- **`keepinventory on|off` cheat:** while on, dying keeps everything the robot carried. The wreck still appears, empty, as a marker of where it happened.
- **`keepbody on|off` cheat:** while on, respawning gives the robot back the same limbs it died with, fully repaired. A missing arm stays missing; parts it had are restored to full integrity.
- Both cheat settings are saved with the world, and they follow the world's "Allow cheats" setting like every other cheat.

## Capabilities

### New Capabilities
- `death-options`: Death markers for wrecks, and the keep-inventory and keep-body options.

### Modified Capabilities
- `cheat-console`: adds `keepinventory` and `keepbody`.

## Impact

- **Core:**
  - `QuestMarkerType::Death` with the wreck's position.
  - The marker is removed when its wreck is empty or gone.
  - `WorldSettings` (or `PlayerState`) gains `keep_inventory` and `keep_body` (saved, default off).
  - `death::player_died` and `Game::respawn` read them.
- **Client:** the marker in the world view and on the Global Map, the delete confirmation, and the two cheats.
- **Depends on:** `add-player-death` (the wreck and respawning).
