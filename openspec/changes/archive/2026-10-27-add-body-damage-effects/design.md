# Design

## Context

- `BodyPart { state, integrity, walk_speed_modifier, break_speed_modifier, health_modifier, stamina_modifier }`. Each state maps to a multiplier only for walking (`get_walk_speed_modifier`).
- `Game::queue_player_move`, `queue_mine` and `queue_mine_at` divide a base cost by `Body::walk_speed_modifier`, so mining follows the legs.
- Melee damage is a fixed 10 in `moves.rs`. The light radius is the `LightSource` component (2, or 8 with a torch). Combat energy is the `Energy` component, which nothing refills except respawning.

## Goals / Non-Goals

**Goals:**
- One effect per part, easy to read.
- Computed in one place.
- Walking stays as today for damaged legs.
- Deterministic.

**Non-Goals:**
- Damage types (physical or electrical), bleeding or overheating.
- Part-specific moves (a kick with legs, a two-handed swing).
- Rebalancing combat numbers beyond the melee factor.

## Decisions

### One function
- `Body::effects()` reads part states and returns a `BodyEffects`.
- **Arms:** each damaged arm multiplies mining time by 1.5 and melee damage by 0.75. One missing arm sets mining time ×2 and melee ×0.6, with the other arm's own state applied on top. Both missing sets `can_mine = false` and `can_melee = false`.
- **Legs:** the weaker leg as today. Missing legs give factors 4 (one) and 8 (both) instead of today's fallback average, so moving never becomes impossible.
- **Head:** a light penalty of 1 when Damaged, and a radius of 1 when Missing.
- **Power source:** energy regen ×0.5 when Damaged.

### Where effects apply
- Movement and mining costs: `base * factor`. Mining uses the arm factor.
- Melee damage: rounded down, at least 1 while melee is possible.
- Light: the radius is clamped to at least 1.
- Energy: there was no regen, so this change adds it. In combat the player recovers 1 energy every 2 ticks (`ENERGY_REGEN_INTERVAL_TICKS`), every 4 with a damaged power source. Out of combat nothing changes.
- With no arms, mining and the melee move are disabled with a message ("No working arm to mine with").

### Showing it
- `BodyEffects::describe()` gives plain lines per active effect, used by the Body tab and, shortened, by the context help.

## Risks / Trade-offs

- [The new-game robot feels sluggish] → It mines at half speed until the arm quest. That is the point, but the user check confirms it isn't frustrating; the constants are in one place.
- [Combat energy regen is new] → It makes fights a little easier for every body; the interval is one constant.
- [Snapshots and scripts that mine or fight change timing] → Reviewed one by one. Scripts using `tick` counts after mining may need more ticks.
