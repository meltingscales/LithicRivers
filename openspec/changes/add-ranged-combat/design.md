# Design

## Context

- **Combat:** the combat trigger marks adjacent creatures with `Combat { triggered }`, and `enemy_combat_ai_system` queues `EnemyAttack` actions with seeded damage and timing.
- **Light:** fog of war casts rays with Bresenham lines for light and walls.
- **Fights stay on one layer**, as in Caves of Qud.

## Goals / Non-Goals

**Goals:**
- A ranged enemy that plays differently: cover and distance matter.
- Limited ammo, so its threat ends.
- Deterministic hit rolls.

**Non-Goals:**
- Player guns (`add-firearms`).
- Projectiles as moving entities. Shots resolve when their action completes; only a short line is drawn.
- Area damage, and 3D combat.

## Decisions

### Line of sight and triggering
- `line_of_sight(a, b)`: the same Bresenham walk as fog of war. Solid tiles block it, and creatures don't.
- A creature with `RangedAttack` and ammo joins combat when the robot is within its range on its layer with line of sight, even if not adjacent.

### Shots
- `EnemyShoot` takes 10 to 14 ticks.
- **Hit chance:** `accuracy_at_1 - falloff * (distance - 1)`, clamped to 10–95%, rolled from `world_state.rng`. A miss logs "The android's shot goes wide".
- **Damage:** 8–14 on a hit, to a random part, like other attacks.
- Each shot uses one ammo. At zero, the android's AI switches to closing in and melee.

### The android
- **Movement:** keeps 4–6 tiles from the robot when it has ammo (steps away if closer, toward if farther, by the dog's pathfinding), and acts every other turn.
- **Stats:** 140 health; melee 6–12 damage every 12–16 ticks.
- **Placement:** an `EnemySpawn` variant in structures picks the android, so ruins can place it. `spawn_enemy military_android` does too.
- **Loot:** its leftover ammo, 1–2 scrap electronics and 1–3 iron scrap.

## Risks / Trade-offs

- [Unfair at range] → Accuracy falls off, walls give cover, and ammo runs out.
- [Fights that never start because of line of sight] → An android that loses sight walks toward the robot's last seen position.
