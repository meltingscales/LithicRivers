# Design

## Context

- `AudioManager` (`crates/client/src/audio.rs`) has a music sink and a sound-effect sink, both scaled by the Options volume sliders. Effects play by name from embedded assets. Headless runs use `AudioManager::disabled()`, which plays nothing.
- Actions happen in two places: core systems (mining, pickup, combat, teleporting) and client input code (placing blocks, crafting).
- Determinism: everything that changes the game must come from the seed and tick. Sound must not feed back into the game.

## Goals / Non-Goals

**Goals:** every common action makes a sound; sounds are our own; nothing about the simulation changes.

**Non-Goals:**
- Positional or stereo sound, and distance falloff.
- Sounds for machines, creatures moving, or ambience.
- Separate volume per cue (the effect slider covers all).

## Decisions

### A cue queue in Resources, drained by the client
`Resources::sounds: Vec<SoundCue>` is `#[serde(skip)]`, and `Resources::play(cue)` pushes onto it. The client calls `play_sound_effect(cue.name())` for each queued cue in `step` and clears the queue. Client-side actions push onto the same queue, so there's one path. The queue is capped (32) so core-only tests and long headless runs can't grow it.
Alternative: the client infers sounds from the message log. Rejected: fragile string matching.

### Where cues come from
- Player mining (`player_movement`): Chop for trees, Mine for anything else, including breaking a factory block. Machines mine through `crafting::mine_tile` too, but cue only from the player path, so a tree farm isn't a constant crackle.
- Pickup: once per pickup that moved items.
- Combat `apply_damage`: Hurt when the target is the player, Hit otherwise.
- `Game::teleport`: Teleport on success.
- Client: Place after a successful block or machine placement, Craft after a successful craft.

### Generated sounds
`scripts/generate_sfx.py` (Python standard library only) writes 22.05 kHz mono 16-bit WAVs into `crates/client/assets/sound/effects/`. Each sound is a short envelope over seeded noise and tones: a gritty burst for Mine, a low thump for Place, a rising blip for Pick up, a sharp crack for Hit, a low buzz for Hurt, a falling sweep for Teleport and a two-note chime for Craft. Fixed seeds make the output byte-identical every run.

### No backlog
`play_sound_effect` skips when the effect sink already holds 3 or more queued sounds. One sink plays sounds one after another, so without the cap a burst of mining would trail on for seconds.
