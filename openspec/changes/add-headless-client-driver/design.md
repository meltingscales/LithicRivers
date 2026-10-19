# Design

## Context

- `run_app<B: Backend>` (`crates/client/src/main.rs`) is already generic over the ratatui backend, so ratatui's in-memory `TestBackend` can render a full frame without a terminal.
- Input enters through one function: `handle_input(app, KeyCode)` (`app/input_handler.rs`).
- Wall-clock reads that affect behavior: splash timers in `App::on_tick` (`Instant::elapsed`), combat auto-tick in `run_app` (`Instant::now` vs `COMBAT_MS_PER_TICK`), hotbar double-tap in `app/input/world_build.rs` (`Instant::now`), and text-effect animation in `dialogue_presenter.rs` (`SystemTime::now`).
- `App::new_with_seed` always opens audio and panics if it can't (`AudioManager` holds a rodio `OutputStream`).
- Keybinds are stored as `"category:ACTION" -> Vec<KeyCode>`. `lithicrivers_core::keycode_mapping::parse_keycode` turns config key names into `KeyCode`s.
- The cheat console exposes a `CommandRegistry` with `execute(app, line)`.
- The client crate is binary-only; `demo_*` bins pull modules in with `#[path]`.

## Goals / Non-Goals

**Goals:**
- One code path for "advance one frame" shared by the interactive loop and the script runner.
- Byte-identical output for the same script, seed and build.
- No new crate dependencies.

**Non-Goals:**
- Mouse input in scripts.
- Color or style information in screen dumps (plain text only; styles can come later if needed).
- New cheat commands such as entity spawning; scripts use the existing registry. Scenario setup beyond `tp`/`giveitem` is follow-up work.
- Loading save files as fixtures (follow-up; `save_load.rs` already supports JSON load).

## Decisions

### Run mode lives in the main client binary, behind `--script <path>`
`main()` checks `std::env::args()` for `--script <path>` (and `--seed`, `--size` overrides) before touching the terminal. A separate `bin/` target would have to `#[path]`-include half the client the way demos do, which duplicates module trees and drifts. clap is not added; two flags don't justify a dependency.

### Extract `step(app, now)` from `run_app`
Everything in the loop except "poll terminal events" and "draw to the terminal" moves into `fn step(app: &mut App, now: Instant)`: combat auto-tick and `on_tick`. The interactive loop becomes `step(app, Instant::now()); terminal.draw(..); poll/handle events`. The script runner calls `step` with fake time and draws into a `TestBackend` only when it needs a screen.

### App-owned clock
Add `Clock` to `App`: `Real` returns `Instant::now()`; `Fake { base: Instant, offset: Duration }` returns `base + offset`, where `base` is captured once at startup. Because behavior only ever uses differences between instants, a fake clock with a real base is fully deterministic. Every behavior-affecting time read listed in Context goes through `app.clock.now()`. `dialogue_presenter` text effects take the animation time in milliseconds as a parameter derived from the clock instead of reading `SystemTime`.
Alternative considered: passing `delta_time` everywhere. Rejected because `on_tick` compares stored `Instant`s against `elapsed()`, so a `now` source is the smaller change.

### Frame cadence under fake time
`wait <ms>` advances the clock in 50 ms frames (the interactive loop's poll interval), calling `step` each frame. This keeps combat auto-ticking and splash transitions identical to real play at the same elapsed time.

### Audio disabled mode
`AudioManager` gets a disabled variant whose `_stream` is `None` and whose play methods are no-ops. `App::new_headless(seed)` uses it; `new_with_seed` is unchanged. This keeps the `AudioState` field type unchanged for the rest of the client.

### Script parsing is validated before execution
The whole file is parsed into a `Vec<Step>` first, so syntax errors and setup-order errors exit 1 before any side effects. Keybind and cheat-name errors are also checked at parse time where possible (keybind lookup uses the loaded config; cheat names use the registry).

### Output format
Dumps go to stdout framed by `=== screen <label> ===` / `=== end screen ===` and `=== state <label> ===` / `=== end state ===`. Labels default to the step's line number. Errors go to stderr. Tracing still writes to `LithicRivers.log`; headless never writes logs to stdout.

### State snapshot
A new `AgentState` struct (client side, `serde::Serialize`) built from `App`, not `SaveData`: `SaveData` includes the full `TileWorld` and chunk maps, which are huge and irrelevant to agents. Struct field order fixes JSON key order; lists are sorted as the spec requires. Nearby radius and list caps are constants (radius 12 tiles, 32 entities, 20 messages). `version: 1` lets the format change later.

### Screen text extraction
After `terminal.draw` into `TestBackend`, read the buffer row by row, concatenate cell symbols, trim trailing spaces. Wide characters occupy one cell plus a skipped cell in ratatui's buffer; the extractor skips cells marked as continuation.

## Risks / Trade-offs

- [Unrouted time read added later breaks determinism] → Add a determinism test that runs a short script twice and compares output; document "use `app.clock`" in AGENTS.md.
- [`HashMap` iteration order leaks into rendering] → The repeat-run test catches it; fix any offender by sorting at the render site.
- [Rendering cost if a script dumps every frame] → Rendering only happens on `screen`/`expect-text` steps, not every `wait` frame.
- [Splash `skip-splash` diverges from real startup] → It sets the same state the splash flow ends in (`SplashState::MainUI`), the same thing pressing through the splash does.

## Migration Plan

Additive. The interactive path changes only by routing time reads through `Clock::Real`. Rollback is reverting the change.
