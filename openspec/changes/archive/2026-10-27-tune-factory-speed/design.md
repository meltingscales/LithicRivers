# Design

## Decision: slow the clock, not each block

Factory systems already measure everything in factory steps (belt 2 steps a tile, extractor 5 steps, smelting `seconds * 5` steps, power in joules per step). Stepping every 60 ticks instead of 20 slows all of them by 3x with one constant and no rebalancing. Retuning each block (option 2) is kept for later tuning per tier; speeding the player up (option 3) touches combat, growth and every creature.

- `FACTORY_SECOND_TICKS = 3 * TICKS_PER_SECOND`, `STEPS_PER_FACTORY_SECOND = 5`, `FACTORY_STEP_TICKS = 300 / 5 = 60`. All integers, so power stays exact.
- Watts are per factory second. A burner generator still reads 90 kW and a log still burns for 22 factory seconds, now 66 game seconds.
- The player's belt ride counts its own steps (`BELT_RIDE_STEPS_PER_TILE = 1`), so riding is barely slower than before while items are 3x slower.
- Belts move in 0.6-second jumps instead of 0.2. A player action is at least 2 game seconds, so this doesn't show in play; the factory demo ticks 120 per beat.

## Risks

- Saves keep `factory_clock.carry` in ticks, so an old save carries a remainder under 60: harmless.
