# Proposal

## Why

Every fight today starts when a creature is next to the robot, and every attack is melee. Feral dogs and nuclear slimes differ only in pace and numbers. The user's idea (IDEAS-INBOX.md) is an enemy that shoots: a rusty military android with limited ammo. That needs ranged attacks in combat first, and it gives the later firearms (`add-firearms`) their counterpart.

## What Changes

- **Ranged combat:**
  - A creature with a ranged attack can start combat from a distance, when it has a clear line of sight to the robot within its range on the same layer.
  - Its shots take time like any queued action, can miss, and are less accurate farther away (accuracy falls off with distance).
  - Walls block line of sight, so cover matters.
- **The rusty military android:** a SapienCorp security unit, mostly broken.
  - It keeps its distance (it prefers 4 to 6 tiles away) and shoots from there.
  - It has **limited ammo** (6 shots, rolled per android from the seed), shown in the combat panel. Once out, it closes in and fights in melee.
  - Sturdier than a dog but slower to act. It appears in ruins and the SapienCorp facility, through the structure spawn tiles.
  - When destroyed it leaves a wreck with its leftover ammo, scrap electronics and iron scrap. The ammo is useless until `add-firearms`.
- **Combat panel:** shows each enemy's distance and, for ranged enemies, their ammo.
- **Cheat:** `spawn_enemy military_android`.

## Capabilities

### New Capabilities
- `ranged-combat`: Line-of-sight ranged attacks with accuracy falloff, combat starting at range, and the rusty military android.

### Modified Capabilities
- `cheat-console`: `spawn_enemy` supports `military_android`.

## Impact

- **Core:**
  - A `RangedAttack { range, ammo, accuracy_at_1, falloff }` component.
  - Line of sight, reusing the fog-of-war raycast.
  - Combat triggering for ranged creatures in range.
  - `CombatAction::EnemyShoot` with a seeded hit roll.
  - The android's AI (keep distance, shoot, close in when out of ammo).
  - Loot through `CreatureLoot`.
- **Client:**
  - Distance and ammo in the combat panel.
  - A shot's line drawn briefly on hit or miss.
  - Android sprites.
- **Depends on:** combat's action queue, and creature loot from `add-nuclear-slime`.
