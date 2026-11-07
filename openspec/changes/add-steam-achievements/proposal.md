# Proposal

## Why

`add-steamworks` connects release builds to Steam with one placeholder achievement (System Failure, for the first death). The Steam MVP lists an "Achievements system" (`docs/STEAM-MVP.md`, Steam integration), and `docs/SPEC-GAPS.md` asks for the real list. Achievements are also a gentle guide: they point new players at the first milestones of the story (escaping the bunker, the two opening quests) and at systems they might miss (transit stations), without saying anything about the lore.

## What Changes

- **Six achievements**, each unlocked once and never in worlds that allow cheats:
  - System Failure (`ACH_FIRST_DEATH`): lose your body. Already exists.
  - Daylight (`ACH_SURFACE`): reach the surface.
  - Self-Defense (`ACH_FIRST_KILL`): destroy an enemy.
  - Spare Parts (`ACH_REPAIR_ANDROID`): complete Repair the Broken Android.
  - Two Hands (`ACH_REPLACE_ARM`): complete the arm quest.
  - Transit (`ACH_FIRST_JUMP`): teleport for the first time.
- **Player stats:** the world counts enemies destroyed and teleports taken, saved with the world. The kill and jump achievements read them, and later achievements can use them too.
- **Checks:** the client checks every condition once a frame, as it does for the first death, and asks Steam for each achievement at most once per session.

## Capabilities

### New Capabilities
- `steam-achievements`: the achievement list, when each unlocks, and that cheats block them.

### Modified Capabilities
<!-- None: player stats are new saved state, covered by save-data's "every Resources field is saved". -->

## Impact

- **Core:** `resources::PlayerStats { enemies_killed, teleports }` in `Resources::stats` (serde default, so old saves load). `handle_entity_death` counts non-player deaths, and `Game::teleport` counts successful jumps.
- **Client:** `steam::Achievement` gains five variants. `App::check_achievements` checks them, and `Steam::unlock` skips achievements already unlocked this session.
- **Steamworks (manual):** add the five new achievements with names and descriptions, and publish.
- **Depends on** `add-steamworks` (#214).
