# Design

## Context

- **Robot death:** `ComponentAccess::apply_damage` marks the player Dead when the power source or torso is destroyed. `check_combat_end_conditions` then ends combat with "Player defeated", and `handle_entity_death` only logs "You have died!".
- **Corpses:** creatures leave an `EntityKind::Corpse` with their inventory, looted through the corpse window (`CorpseLootingState`).
- **New game:** the player spawns at the bunker's `special_player_spawn` block, with a starting body (left arm missing, a damaged leg) and the starting kit.

## Goals / Non-Goals

**Goals:** a clear, recoverable outcome; nothing the player carried is lost forever; deterministic; survives save and reload at any point.

**Non-Goals:**
- The real backup body: a body that's worse than the starting one, or built somewhere (the user's call, later).
- Salvaging parts from the wreck (the wreck keeps the body so a later change can).
- Permadeath or difficulty settings (later, `docs/SPEC-GAPS.md` "Difficulty templates").
- Moving the respawn point in play (a later change).
- Death from anything other than body damage (no hunger, falling or radiation yet).

## Decisions

### The wreck
- A corpse entity with `EntityKind::Corpse`, a `Wreck` marker (saved), the player's whole inventory and a copy of the player's `Body`, at the death position. It uses its own sprite so it reads as the player's old body.
- Looting it uses the existing corpse window. An empty wreck stays as scenery: it is a body, not a bag.
- **Death has two paths today:** losing the power source or torso in a fight (`ComponentAccess::apply_damage`), and `handle_entity_death` (debug kill). Both reach one function that makes the wreck, so it can't be missed.

### Respawning
- The player entity is kept, not respawned, so everything that refers to it stays valid.
- Respawning:
  1. clears `Dead`, combat and the action queue
  2. sets the default body and the starting kit
  3. moves the player to `respawn_point`
  4. snaps the view there
- The starting kit is built by the same function a new game uses, so the two can't drift.

### The screen
- `PlayerState::combat_active` is already false after a defeat. The client shows the modal whenever the player is Dead. That is derived state, so a save made while dead shows the screen again on load.
- The modal lists the cause from the last damage message ("Fusion Core destroyed by a feral dog").

### Testing death
- A `kill_player` cheat destroys the player's body on demand, for scripts and play-testing (`scripts/snapshots/player-death.txt`).
- Respawning runs one game tick, so light and fog of war catch up with the new position before the next frame.

## Risks / Trade-offs

- [Dying near the respawn point loops] → The player gets a `BattleDelay` when respawning, the same rule as escaping, so a creature waiting there can't attack at once.
- [Free healing by dying] → The respawned body is only the starting body, items stay at the wreck, and the walk back is the cost. The real backup body can make it costlier later.
