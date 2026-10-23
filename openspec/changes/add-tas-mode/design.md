# Design

## Context

- **Determinism:** worldgen, combat, loot and AI already read only the seed, the world tick and coordinates (`openspec/config.yaml`).
- **Headless client:** `just client-blind` runs scripts against a fake clock in 50 ms frames, and the MCP session can export what it did as a script.
- **Wall-clock leaks in the live client:** combat auto-ticks use `last_combat_tick: Option<Instant>`; anything else that reads `Instant::now()` breaks replay.
- **Saves:** `save-reload` already saves and loads the whole game, client play state included, in memory.

## Goals / Non-Goals

**Goals:**
- A hand-played TAS run replays to the same state in the live client and in `just client-blind`.
- Movies are plain, editable text in the existing script format.
- Savestates and rerecords like other TAS tools.

**Non-Goals:**
- Slow motion at a real-time rate, an input piano roll, or a movie editor in the game.
- Leaderboards, verification servers or Steam integration.
- A defined end of the game (see the proposal).

## Decisions

### Time in a TAS world
The live client gets the same frame-based clock the headless client uses. In a TAS world `app.clock` only moves when `tas.FRAME_ADVANCE` is pressed (one frame) or held (one frame per key repeat). Outside TAS worlds the live client keeps reading real time. Combat auto-ticks move from `Instant` to `app.clock` in both, which also fixes the leak for headless runs.

### Movie format
A movie is a headless script, so there is one format and one player:
- A header of `seed`, `size` and `world <type>` (or `skip-splash`) from the world's settings, plus a `# lithicrivers <version> <commit>` comment, since replay only holds for one build.
- Each input becomes the same step the headless runner accepts (`key`, `rawkey`, `click`, `scroll`, `move`, `type`), and each frame advance becomes `wait 50`, with runs of waits merged.
- The movie lives at `<world>/tas/movie.txt` and is rewritten as it grows, so a crash loses at most the last frame.

The recorder sits where input is dispatched, so it records what the game acted on, not raw terminal events. The MCP session's `export_script` uses the same recorder.

### Savestates
Four slots (`tas.SAVESTATE_1` to `_4`, `tas.LOADSTATE_1` to `_4`). A savestate stores the in-memory save and the movie's length at that frame. Loading one restores the game, truncates the movie to that length and adds one to the rerecord count, which is saved with the world.

### Playback and the state hash
Play movie copies the world's starting settings into a temporary world, runs the movie through the headless runner and compares a hash of the final save with the hash written at the end of the movie (`# end-state <hash>`). The hash covers the core save data, not the client view, so resizing the terminal doesn't break it. `just client-blind` gains an `expect-state-hash <hash>` step for the same check.

### Cheats
TAS mode doesn't change Allow cheats; a movie that uses cheats records them as `cheat` steps. The world list shows both flags.

## Risks / Trade-offs

- **Hidden wall-clock reads** would break replay. Mitigation: a test that plays a recorded movie twice in the live-client code path and compares state hashes, and a grep check for `Instant::now()` outside the clock.
- **Build-locked movies:** any code change can desync a movie. That is normal for TAS tools; the header names the build.
