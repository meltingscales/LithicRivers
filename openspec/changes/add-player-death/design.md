# Design

## Context

- **Robot death:** `ComponentAccess::apply_damage` marks the player Dead when the power source or torso is destroyed. `check_combat_end_conditions` then ends combat with "Player defeated", and `handle_entity_death` only logs "You have died!".
- **Corpses:** creatures leave an `EntityKind::Corpse` with their inventory, looted through the corpse window (`CorpseLootingState`).
- **New game:** the player spawns at the bunker's `special_player_spawn` block, with a starting body (left arm missing, a damaged leg) and the starting kit.

## Goals / Non-Goals

**Goals:** a clear, recoverable outcome; nothing the player carried is lost forever; deterministic; survives save and reload at any point.

**Non-Goals:**
- Permadeath or difficulty settings (later, `docs/SPEC-GAPS.md` "Difficulty templates").
- Moving the backup point in play (a later change).
- Death from anything other than body damage (no hunger, falling or radiation yet).

## Decisions

### The wreck
- A corpse entity with `EntityKind::Corpse`, a `Wreck` marker (saved) and the player's whole inventory, at the death position. It uses its own sprite so it reads as the player's old body.
- Looting it uses the existing corpse window. An empty wreck stays as scenery: it is a body, not a bag.

### Booting the backup body
- The player entity is kept, not respawned, so everything that refers to it stays valid.
- Booting:
  1. clears `Dead`, combat and the action queue
  2. sets the starting body and kit
  3. moves the player to `backup_point`
  4. snaps the view there
- The starting body is built by the same function a new game uses, so the two can't drift.

### The screen
- `PlayerState::combat_active` is already false after a defeat. The client shows the modal whenever the player is Dead. That is derived state, so a save made while dead shows the screen again on load.
- The modal lists the cause from the last damage message ("Fusion Core destroyed by a feral dog").

## Risks / Trade-offs

- [Dying near the backup point loops] → Creatures near the backup point get a short `BattleDelay` when the backup body boots, using the same rule as escaping.
- [Free healing by dying] → The backup body is only the starting body, items stay at the wreck, and the walk back is the cost. If that proves too cheap, the backup body can start more damaged.
