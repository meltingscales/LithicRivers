# Proposal

## Why

The Steam MVP's survival loop is explore, fight, take damage, repair. The body panel, part damage and repairs exist, but the MVP asks for damage with a clear mechanical consequence, and today there's almost none:
- **Legs** do slow walking (`Body::walk_speed_modifier`). Mining reuses the same leg-based number, so a robot with a broken leg also mines slower, while a missing arm changes nothing.
- **Unused fields:** every part has `break_speed_modifier`, `health_modifier` and `stamina_modifier` fields that nothing reads.
- **Nothing tells the player** what their damage is doing to them.

If damage matters, repairs and the arm-replacement quest (`add-arm-replacement-quest`) matter, and fights have stakes beyond death (`add-player-death`).

## What Changes

One clear effect per part, named in the Body tab:

| Part | Damaged | Missing |
|---|---|---|
| **Legs** (the weaker leg) | Walking takes twice as long (as today) | One leg: walking takes four times as long. Both: the robot drags itself (eight times) |
| **Arms** | Mining and breaking take 1.5 times as long, and melee hits 25% softer, per damaged arm | One arm: mining and breaking take twice as long, and melee hits 40% softer. Both: no mining or melee; tackle still works |
| **Head** | The light radius shrinks by 1 tile | The light radius is 1 tile |
| **Power source** | Energy recovers half as fast in combat | (Losing it kills the robot, as today) |
| **Torso** | No effect by itself | (Losing it kills the robot, as today) |

- **Mining uses the arms**, not the legs.
- **The Body tab lists each active effect in plain words**, for example "Left arm missing: mining 2x slower, melee -40%". The context help on the World tab shows a short warning when an effect applies.
- **Effects follow part state** (Missing, Damaged, Functional, Enhanced), not exact integrity, so they're easy to read and to fix by repairing. Enhanced parts keep today's bonus where one exists.
- **The new-game robot** (left arm missing, a damaged leg) feels both from the start, which is what the arm quest promises to fix.

## Capabilities

### New Capabilities
- `body-damage-effects`: What each part's damage does to walking, mining, melee, light and energy, and how the Body tab shows it.

### Modified Capabilities
<!-- None in openspec/specs: body parts have no spec yet. -->

## Impact

- **Core:**
  - `Body::effects() -> BodyEffects { walk_factor, mine_factor, melee_factor, light_penalty, energy_regen_factor }`, computed from part states. It replaces the scattered `walk_speed_modifier` use.
  - `queue_mine_at` and melee damage use the arm factors. The light source reads the head penalty, and combat energy regen reads the power source factor.
- **Client:** the Body tab's effects list and the context-help warning.
- **Tests:**
  - each row of the table, and Enhanced
  - mining no longer slowed by a damaged leg
  - deterministic replays
- **Tuning:** the numbers are constants in one place. Today's walking behavior stays the same for damaged legs.
