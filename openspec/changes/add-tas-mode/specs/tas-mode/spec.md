# Spec Delta

## Purpose

TAS worlds: input-driven time, movie recording, savestates and playback, so speedrunners can script and replay runs.

## ADDED Requirements

### Requirement: TAS worlds
The Create New World form SHALL offer a TAS mode toggle, off by default. A world created with it SHALL stay a TAS world, and the world list SHALL show that it is one.

#### Scenario: Create a TAS world
- **WHEN** the player turns TAS mode on and creates a world
- **THEN** the world's settings record TAS mode, and the world list marks it

### Requirement: Input-driven time
In a TAS world, nothing SHALL advance on the wall clock: the client clock SHALL move only by frame advances, one 50 ms frame per press.

#### Scenario: Combat waits for the player
- **WHEN** a TAS world is in combat and no key is pressed for a real minute
- **THEN** the game tick and the combat state don't change

#### Scenario: Frame advance
- **WHEN** the player presses the frame-advance key
- **THEN** the client clock moves one frame and any auto-tick due on that frame happens

### Requirement: Movie recording
A TAS world SHALL record every input and frame advance to a movie file in its folder, in the headless script format, and the headless client SHALL replay that file to the same final state.

#### Scenario: Replay a hand-played run
- **WHEN** the player plays a TAS world, and the movie is then run with `just client-blind`
- **THEN** the run ends with the same state hash as the live game

### Requirement: Savestates
A TAS world SHALL offer numbered savestate slots. Loading a slot SHALL restore the game to that frame, cut the movie back to it and add one to the rerecord count.

#### Scenario: Rerecord
- **WHEN** the player saves state in slot 1, plays on, and loads slot 1
- **THEN** the game and the movie are back at slot 1's frame, and the rerecord count is one higher

### Requirement: Playback check
Playing a TAS world's movie SHALL report whether it ends in the state the movie recorded.

#### Scenario: Desynced movie
- **WHEN** a movie's steps are edited so the run ends differently
- **THEN** playback reports a state mismatch

### Requirement: Checking a state hash in scripts
The headless client SHALL accept an `expect-state-hash <hash>` step that fails the run unless the current state hash matches.

#### Scenario: Mismatch
- **WHEN** a script's `expect-state-hash` names a different hash than the game's
- **THEN** the run fails with the expected and actual hashes
