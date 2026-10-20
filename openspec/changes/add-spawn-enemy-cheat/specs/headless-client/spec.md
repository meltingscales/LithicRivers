# Spec Delta

## MODIFIED Requirements

### Requirement: Time and tick steps
`wait <ms>` SHALL advance the client's clock by that many milliseconds in frame-sized increments, running per-frame updates (splash timers, combat auto-ticks) as the interactive loop would. `tick [n]` SHALL advance the game simulation by `n` ticks (default 1) and SHALL apply each tick's combat transitions to the client the same way a player action does: combat starting opens the combat panel on the World tab, and combat ending closes it. No step SHALL depend on real elapsed time.

#### Scenario: Combat auto-tick under fake time
- **WHEN** combat is active, `COMBAT_MS_PER_TICK` is 1000, and the script runs `wait 3000`
- **THEN** the game advances exactly three combat ticks, however long the run takes in real time

#### Scenario: Combat starts during a tick step
- **WHEN** a feral dog reaches the player during a `tick` step
- **THEN** the state dump reports combat active and the screen shows the combat panel, as it would after a player action that took the same tick
