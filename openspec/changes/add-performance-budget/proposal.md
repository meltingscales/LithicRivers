# Proposal

## Why

The Steam MVP plan sets a performance target (about 60 FPS at 80x24, at most 16 ms per tick) but nothing measures it. There is no benchmark, no recorded baseline, and the only profiling recipes (`just profile-client-*`) run `clean-linux` first, profile a debug build, and need someone to play the interactive client by hand. Steam Deck QA and the December launch need a number that can be checked, and a guard so later changes don't quietly undo the fixes.

## What Changes

- **The budget, written down:** a game tick (one player action, including a 1000-tick Pass Time) finishes in at most 16 ms, and a frame (one client step plus drawing the screen) in at most 16 ms at 80x24. Both are measured on a release build with a fixed set of scenarios.
- **A benchmark:** `just util-bench` runs the scenarios headless on a release build and prints tick and frame times (median, 95th percentile, worst) against the budget. It exits non-zero when a scenario is over budget. The scenarios are ordinary headless scripts, so the work is identical on every run: walking the surface, Pass Time at night, a busy factory (the factory showcase world) and the deep underground.
- **Profiling that works again:** a recipe that profiles one of those scripts on a release build with debug symbols, with no clean step and no interactive client. It writes a flamegraph and a file speedscope.app opens. The old `profile-client-*` recipes are replaced.
- **Fix the top two hotspots** the profiles show, each with a test that guards it by counting work (deterministic and safe for CI), not by timing.
- **Steam MVP doc:** record the baseline and the results in `docs/STEAM-MVP.md` and tick the performance items.

## Capabilities

### New Capabilities
- `performance-budget`: the tick and frame budget, the benchmark scenarios and their report, and the profiling recipe.

### Modified Capabilities
None. The hotspot fixes change how fast things run, not what they do. The headless scripts and snapshots must print the same output before and after.

## Impact

- New `just util-bench` and `just util-profile <script>` recipes. The old `profile-client-linux` / `profile-client-windows` / `profile-setup` recipes are replaced.
- A Cargo profile for profiling (release with debug info), in the workspace `Cargo.toml`.
- A bench entry point in the client crate that drives the headless runner and times ticks and frames with the real clock. Game logic still reads time only from `app.clock`, so script output stays deterministic.
- New scenario scripts under `scripts/bench/`.
- Core and client code wherever the two hotspots turn out to be. Candidates are listed in design.md.
- No new runtime dependencies. Timing uses `std::time::Instant`, only in the bench entry point.
