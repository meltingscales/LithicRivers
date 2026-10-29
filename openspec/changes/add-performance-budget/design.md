# Design

## Context

- The game is turn based. A tick happens when the player acts (`Game::tick`, with the action's cost added to the clock), during combat on a timer, and as one call for Pass Time (1000 ticks). Each tick runs the system scheduler, the due factory steps, quests and death markers, and first `load_around_player`, which visits the 27 chunks around the player and logs two `tracing::info!` lines for each.
- The interactive loop (`run_app` in `crates/client/src/main.rs`) runs `step`, draws, then waits up to 50 ms for input. So idle redraws are capped at 20 per second. "60 FPS" here means a frame costs at most 16 ms, so the game could draw 60 per second. It does not mean redrawing 60 times a second while idle.
- The headless runner already reproduces that loop: `press` is `handle_input` (which ticks for world actions) followed by `frame` (`step` + draw into a `TestBackend`). Scripts are deterministic and use a fake clock.
- The current profiling recipes depend on `clean-linux`, profile `target/debug`, and need the interactive client. Agents can't run that, and you have to play it by hand.

## Goals / Non-Goals

**Goals:**
- One number per scenario that anyone can reproduce: tick and frame times on a release build.
- Profiling that an agent can start from a script.
- Fixes for the two biggest hotspots, each guarded without timing-based tests.

**Non-Goals:**
- Raising the idle redraw rate, or changing the 50 ms input poll.
- Profiling on Windows or macOS. The profiling recipe is Linux only (perf); the benchmark itself runs anywhere.
- Running the timed benchmark in PR checks. Shared CI runners are too noisy for a 16 ms threshold.
- Optimizing past the budget, or hotspots beyond the top two.

## Decisions

**Time the headless runner, not the core.** The bench mode wraps the runner's `press` and `frame`. It records an *update* time (input handling plus `step`, which contains any tick) and a *draw* time for every frame. The update time is reported as the tick time. It is slightly larger than `Game::tick` alone, so the budget errs on the strict side, and core needs no timing code. The alternative, timers inside `Game::tick`, would put `Instant` into core, which must stay free of real time.

**Scenarios are scripts, with `beat` marking where setup ends.** Steps before the first `beat` (start the world, teleport, give items, build) are not timed. That reuses the demo-mode convention, where steps before the first `beat` set the scene, so no new script step is needed. Scenarios live in `scripts/bench/`, use `size 80x24`, and are plain scripts, so `just client-blind` runs them too. A test runs each one headless to keep it valid. The first set:
- `surface-walk`: walk 200 steps across the starting surface, exploring new chunks.
- `night-pass-time`: Pass Time repeatedly through a surface night (fog of war, creatures).
- `factory-busy`: the factory showcase world with machines running, plus Pass Time.
- `deep-underground`: walk and mine in the Lithic Rivers depths.

**A `--bench <script>...` flag on the client binary.** It sits next to `--script` and `--mcp` in `run_headless_from_args`, reuses the runner, and prints the report table. `just util-bench` runs it with `cargo run --release` over `scripts/bench/*.txt`. A separate binary would duplicate the argument handling and the runner setup. The report prints median, 95th percentile and worst times, because the worst value is what a player feels and what the budget checks. The exit status is 1 when any scenario is over budget.

**Profiling with perf and inferno, on a `profiling` Cargo profile.** `[profile.profiling]` inherits `release` and adds `debug = true`. `just util-profile <script>` builds it, then runs `perf record --call-graph dwarf` on the client with `--script <script>`. It writes `profile.linux-perf.txt` (`perf script` output, which speedscope.app imports) and `flamegraph.svg` (`inferno-collapse-perf | inferno-flamegraph`). `just util-profile-setup` replaces `profile-setup` and installs inferno. The recipe checks for `perf` and `kernel.perf_event_paranoid` first and stops with a clear message, rather than asking for sudo by itself. We considered cargo-flamegraph, but it hides the perf data needed for the speedscope file, and the old recipes no longer used it anyway. `tracing-chrome` would add a dependency and only see code that has spans.

**Guard fixes by counting work.** Each fix gets a test that counts the redundant work it removed, through a counter visible to tests, for example "chunks generated or scanned during 100 ticks of standing still" or "full entity scans per rendered frame". The test asserts the count stays at the fixed level. The counter lives behind `cfg(test)` or the core `test-support` feature, and adds no cost to release builds. Timing assertions were rejected because they are flaky on CI.

**Hotspots are chosen from the profiles, not guessed.** The candidates to check first are:
- `load_around_player` running 27 chunk checks and 54 `info!` calls on every tick, even when the player hasn't changed chunk.
- `build_render_view` querying every entity in the world each frame instead of those in view.
- Factory steps during a 1000-tick Pass Time.
- Fog-of-war and lighting recomputed for the whole view each frame.

The two that cost the most across the scenarios get fixed, and the profile that shows each is summarized in the PR.

## Risks / Trade-offs

- [Machine-dependent numbers] → The report prints the build profile and CPU model. The baseline in `docs/STEAM-MVP.md` names the machine. The budget is checked on the developer's machine, and the Steam Deck stays a manual QA item.
- [Bench timing changes output] → Timing only reads `Instant` inside the bench mode and never feeds back into the app. The existing snapshot and `save_reload_anywhere_changes_nothing` tests prove that script output is unchanged.
- [A fix changes behavior, for example skipping chunk loads changes when structures appear] → Every snapshot script must pass unchanged, and `game_new_matches_golden_entities` must still match.
- [perf not available, for example in WSL or containers] → The recipe stops with an explanation, and the benchmark still works without it.

## Migration Plan

Developer tooling only. The old `profile-client-linux`, `profile-client-windows` and `profile-setup` recipes are removed in the same change, and docs that mention them are updated. Nothing in saves or config changes.
