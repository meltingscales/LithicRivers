# Proposal

## Why

LithicRivers is deterministic from the seed and the inputs by design, so a run can be replayed exactly. The user's idea (IDEAS-INBOX.md): a "TAS mode" on the Create New World page that makes it easier for speedrunners to beat the game programmatically. The pieces already exist for agents (headless scripts, the fake clock, `export_script` over MCP), but a player can't reach them from the game, and the live client still reads the wall clock for combat auto-ticks, so a hand-played run can't be replayed.

## What Changes

- **A TAS mode toggle** on the Create New World form (off by default). A TAS world is marked as one in the world list and its save, and the setting can't be changed later.
- **Input-driven time:** in a TAS world nothing advances on the wall clock. Combat auto-ticks, blinking and other clock-driven behavior advance only when the player presses a frame-advance key (one 50 ms frame) or holds it.
- **Recording:** every input is written, frame by frame, to a movie file in the world's folder. The movie is a headless script (`seed`, `world`/`skip-splash`, then `key`, `click`, `wait` steps), so `just client-blind` plays it and a speedrunner can write or edit one in a text editor.
- **Savestates:** a few numbered savestate slots (in memory, plus written with the world). Loading one rewinds the movie to that frame and counts a rerecord.
- **Playback:** "Play movie" on the world's actions plays a movie file into a fresh copy of the world and reports whether it ends in the same state (a hash of the save).
- **A TAS HUD:** frame count, game tick, rerecord count and input count.
- **A guide,** `docs/TAS.md`: the movie format, keybinds and how to drive a run from the headless client or MCP.

## Capabilities

### New Capabilities
- `tas-mode`: TAS worlds, input-driven time, movie recording, savestates and playback.

### Modified Capabilities
<!-- None: the Create New World row, Play movie and the `expect-state-hash` script step are new requirements in `tas-mode`; existing title-menu and headless-client behavior doesn't change. -->

## Impact

- **Core:** `WorldSettings::tas_mode`; a state hash over the save data.
- **Client:**
  - The time source behind `app.clock`: in a TAS world it is the frame counter, never `Instant::now()` (combat auto-ticks today use `last_combat_tick: Instant`).
  - An input recorder that writes script steps, shared with the MCP session's `export_script`.
  - Savestate slots built on the in-memory save used by `save-reload`.
  - The TAS keybinds (`tas.FRAME_ADVANCE`, `tas.SAVESTATE_<n>`, `tas.LOADSTATE_<n>`) in `default_config.rs`.
- **Out of scope:** a win condition and run timer end point. "Beating the game" isn't defined yet; the HUD counts frames from world creation, and a later change can stop the timer at the final milestone.
