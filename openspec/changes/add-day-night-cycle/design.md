# Design

## Context

- `GameTime { tick }`, with 100 ticks per game second (`factory::TICKS_PER_SECOND`).
- `fog_of_war.rs` illuminates every tile at z ≥ 0 and runs light raycasting below that, with the player's `LightSource` (2 tiles, 8 with a torch).
- `feral_dog_system` chases within 15 tiles.
- Scripts and snapshots mostly play near tick 0 on the surface.

## Goals / Non-Goals

**Goals:** a readable rhythm; deterministic from the tick; cheap (no per-tile light map on the surface by day); old saves keep working.

**Non-Goals:**
- Placed light sources (lamps), weather, seasons, and crops or trees growing by daylight (later changes).

## Decisions

### Time of day
- `DAY_TICKS = 90_000`. `time_of_day = (tick + START_OFFSET) % DAY_TICKS`, with `START_OFFSET` putting tick 0 at 08:00, so new worlds and the first minutes of every script are in full day.
- **Phases:** night 21:00 to 05:00, dawn 05:00 to 07:00, day 07:00 to 19:00, dusk 19:00 to 21:00.

### Light on the surface
- By day, the surface is fully lit as today. At night, fog of war on the surface works as underground.
- Dawn and dusk are drawn, not simulated: the client darkens remembered and unlit colors by a factor that eases between day and night, and fog of war switches on at the start of night.
- The light radius doesn't change with the phase. The torch is how the player pushes back the night.

### Dogs
- The notice radius becomes `if night { 22 } else { 15 }`. Their other behavior is unchanged.

### Determinism and saves
- The time of day follows the tick, so it saves and replays with no new state.
- `set_time` advances the tick to the next matching time instead of moving it backward, so tick-based timers stay ordered.

## Risks / Trade-offs

- [Snapshots shift when scripts pass a lot of time] → The 08:00 start keeps short scripts in daylight. Scripts that pass hours are reviewed and can use `set_time 12:00`.
- [Night is annoying when building] → A torch gives 8 tiles of light. A later change can add lamps.
