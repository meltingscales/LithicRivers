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

## Implementation Notes

- **Playback:**
  - `app::demo` holds a `DemoState` on `App`. `step()` (the frame update shared by the interactive loop and headless runs) calls `update_demo`.
  - Scripted keys go through `input_handler::dispatch`. While a demo plays, `handle_input` ignores the player's keys except `title.BACK`, which ends it.
  - Cheats run through the command registry, and `wait` holds playback for its time.
- **New script steps:**
  - `world <type>` starts a debug world without saving. Create New World would write to the saves folder.
  - `beat <ms>` marks where a scenario's shown part starts and sets its pace. Both work in ordinary scripts too, where `beat` is a no-op.
- **Nothing is saved:** `save_world` refuses while a demo plays, and leaving resets to an empty game, like quitting to the title.
- **Scenarios** (`crates/client/assets/demos/`), in order. Each runs about 9–15 s, within the 20 s limit, and the loop takes about 87 s.
  1. **Waking up in the bunker:** fog of war; then the surface, 2x and 3x zoom, and the Global Map.
  2. **Chop, craft, look it up:** Break mode, the Crafting tab, an Item Dictionary search.
  3. **A feral dog attacks:** turn-queue combat with melee and a fireball, and the body panel.
  4. **The dog arena:** 100 dogs closing in.
  5. **Building a belt line:** Place mode, the hotbar, rotation, a log riding into a chest.
  6. **The factory showcase:** riding a belt, the extractor line and router, a lit furnace window, a filter picker.
  7. **A self-feeding tree farm:** the pole showing 180 kW of supply against 240 kW of demand, then 2x zoom while saplings grow and fall.
- **Only feral dogs exist as enemies,** so "different enemies" is shown as a single fight and the 100-dog arena; a new enemy waits for `add-nuclear-slime`.
- **Determinism fix found by the arena:** `SystemScheduler::compute_execution_order` built its dependency graph in `HashMap` order. Systems that depend on the same system could run in either order, so enemy attacks were sometimes queued a tick earlier. It now walks systems by (phase, name), and `execution_order_is_the_same_for_every_scheduler` checks it across 50 schedulers. No demo-only combat mode was needed.
- **Title menu:** Watch Demo is row 1 (`WATCH_DEMO`), so Options moved to row 2. `options.txt` and the options test press Down twice.
