# Design

## Context

- `add-steamworks` added `steam::Achievement` (one variant), `Steam::unlock` (records into `unlocked_this_session`, then sets and stores in Steam when connected) and `App::check_achievements`, which runs every frame and unlocks System Failure on the alive-to-dead edge unless the world allows cheats.
- Quest completion is in `Resources` (`is_quest_completed(QuestType)`). Enemy deaths go through `systems::handle_entity_death`, and every teleport (teleporter, station or device) goes through `Game::teleport`.

## Goals / Non-Goals

**Goals:** a first real list that follows the opening of the game; conditions that are easy to test without Steam.

**Non-Goals:**
- Progress stats in Steam (Steam's stat API) or achievement progress bars.
- Achievements that reveal the endgame or the lore (`docs/QUESTS.md`: the goal stays hidden).
- Retroactively unlocking from old saves beyond what their state already shows.

## Decisions

### State checks, not events
Each achievement except the first death is a condition on the current state: the player's z, the stats counters, completed quests. The client checks them every frame. That needs no event plumbing, and loading an old world that already meets a condition unlocks it, which is fine since the player did earn it. The first death stays an edge, because a world saved on the system-failure screen shouldn't count.

### Counters live in Resources
`PlayerStats` is a small serde struct in `Resources`, so it's saved with the world, and old saves load with zeros. Counting happens in core, at the single places deaths and jumps already go through. Counters, not booleans, so later achievements (destroy 100 enemies) need no new state.

### Unlock once per session
`Steam::unlock` returns early when the achievement is already in `unlocked_this_session`, so state checks that stay true don't call Steam every frame. Steam itself ignores repeats across sessions.

### Cheats
`check_achievements` returns before checking anything when the world allows cheats. This already holds for System Failure.
