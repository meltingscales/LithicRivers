# Design

## Context

- **Headless scripts** (`headless.rs`) parse steps such as `key`, `cheat`, `tick`, `wait` and `type`, and run them against a real `App` with a fake clock.
- **Debug world types** (`add-debug-worlds`) give ready-made scenes: Factory showcase, Dog arena, Void.
- **The title menu** is `SplashState::Title` with `TITLE_CHOICES`.
- **`app.clock`** drives all time-dependent rendering, so playback is deterministic.

## Goals / Non-Goals

**Goals:** short, real, deterministic gameplay cuts behind a banner; scenarios that can't silently break.

**Non-Goals:** recording a human's play as a demo, an idle timer that starts the demo by itself (later), sound design for the demo, video export.

## Decisions

### Scenarios are scripts
- Each scenario is a headless script in `crates/client/assets/demos/` (`explore.txt`, `combat.txt`, `build.txt`, `factory.txt`), with a header line naming it.
- The demo player parses it with the existing parser and runs one step per beat (about 150 ms of `app.clock`). `wait` and `tick` steps run as long pauses, so a factory scenario can let the world run.
- A scenario starts from a fresh `Game` (its own seed and world type) in a throwaway app state that never sets a world folder, so nothing is saved.

### Cuts
- Scenarios play in a fixed order and loop.
- Each one ends when its script ends or after 20 seconds, whichever comes first.
- A cut is a one-beat blank frame with the next scenario's name, like a hard cut on a CRT.

### Banner
- The top row is red with black bold text: "DEMO   " repeated. It shifts one column every 100 ms of `app.clock`, and wraps.
- The scenario name sits right-aligned on the same row, over a black background.
- The world view, panels and messages render as normal underneath, so the demo shows the real UI.

### Input
- In `SplashState::Demo`, `title.BACK` (Escape) returns to the title screen with Watch Demo selected. Every other key is ignored.

### Keeping scenarios working
- Every demo script also runs as a snapshot test (from `crates/client/assets/demos/`), so a change that breaks a scenario, for example a moved block or a renamed keybind, fails `cargo test`.

## Risks / Trade-offs

- [Scenarios drift as the game changes] → Snapshot tests catch it; the scripts are short.
- [Combat can end early or go badly] → The combat scenario uses `spawn_enemy` next to the player and a fixed seed, so it always plays out the same way.
