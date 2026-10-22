# Proposal

## Why

The Steam MVP lists a day/night cycle as a high-priority gap in the core loop: about 10 to 20 minutes per day, with a visual cue. Today the surface is always fully lit, so there is no rhythm to a day and no reason to go underground, build lights or head home. Night also gives fog of war and torches (which already work underground) a job on the surface.

## What Changes

- **A day of 15 game minutes** (90,000 ticks), counted from the world's tick, so the time of day is the same for the same world and moment: dawn, day, dusk and night.
- **Night on the surface (z ≥ 0):**
  - Fog of war applies at night as it does underground: the robot sees its light radius (2 tiles, 8 with a torch), and lit areas it has seen stay remembered in grayscale.
  - Dawn and dusk shade the view between full light and night.
  - Underground layers are unchanged: always dark beyond the light.
- **A visual cue:** the MODES panel shows a clock glyph and the time (for example "☼ 09:40" by day and "* 22:10" for stars at night; `docs/ART-DIRECTION.md` allows CP437 characters only), and the title line can tint to match.
- **Feral dogs are bolder at night:** they notice the player from farther away (15 tiles by day, 22 at night). This is the only gameplay difference besides the dark.
- **Passing time:** the pass-time key and machines keep working the same; a day passes in 90 pass-time presses.
- **Cheat:** `set_time <hh:mm>` sets the time of day for testing.

## Capabilities

### New Capabilities
- `day-night-cycle`: The length of a day, its phases, surface night lighting, the clock in the MODES panel, and night behavior for dogs.

### Modified Capabilities
- `cheat-console`: adds `set_time`.

## Impact

- **Core:**
  - `GameTime::time_of_day()` and phase helpers (no new saved state: it follows the tick).
  - The fog-of-war system lights the surface only by day.
  - The dog notice radius reads the phase.
- **Client:** the MODES panel clock, and dawn and dusk shading in the world view.
- **Tests:**
  - phases by tick
  - surface fog at night and none at noon
  - underground unchanged
  - the dog notice radius
  - `set_time`
  - snapshots at noon and at midnight
- **Existing snapshots:** new worlds start at 08:00 so a whole morning of play stays lit, and most existing scripts run in daylight and don't change.
