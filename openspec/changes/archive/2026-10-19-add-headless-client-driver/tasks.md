# Tasks

## 1. Clock and frame step

- [x] 1.1 Add `Clock` (`Real` / `Fake`) to `App`, defaulting to `Real`; verify `cargo test --workspace` passes and `just client-linux` behaves unchanged (user check)
- [x] 1.2 Route splash timers in `App::on_tick`, hotbar double-tap in `world_build.rs`, and dialogue text-effect animation time through `app.clock`; verify no `Instant::now`/`SystemTime::now` remains in client gameplay code outside `Clock` with `grep -rn 'Instant::now\|SystemTime::now' crates/client/src --include='*.rs' | grep -v bin/`
- [x] 1.3 Extract `step(app)` (time read from `app.clock`) from `run_app` (combat auto-tick + `on_tick`) and have `run_app` call it; add a unit test that, with a fake clock in active combat and `COMBAT_MS_PER_TICK`=1000, advancing 3000 ms in 50 ms frames ticks the game exactly 3 times

## 2. Headless app construction

- [x] 2.1 Add a disabled `AudioManager` mode with no output stream and no-op playback; verify a unit test constructs it without an audio device
- [x] 2.2 Add `App::new_headless(seed)` using disabled audio and `Clock::Fake`; verify a test builds it and calls `step` without panicking

## 3. Script parsing

- [x] 3.1 Implement the step parser (`seed`, `size`, `skip-splash`, `key`, `rawkey`, `type`, `wait`, `tick`, `cheat`, `screen`, `state`, `expect-text`, `quit`, comments, blank lines) with line-numbered errors; verify unit tests for each command and for unknown command, bad arguments, and setup-after-gameplay errors
- [x] 3.2 Validate keybind names against the loaded keybind map and cheat names against the command registry at parse time; verify tests for `key movement.FLY` and an unknown cheat both fail with the line number

## 4. Runner, screen dumps, expectations

- [x] 4.1 Implement the runner executing steps against a headless `App` and a `TestBackend` of the configured size; verify a test runs `skip-splash` + a movement keybind (`movement.MOVE_SOUTH`; north of the seed-12345 spawn is a wall) and the player position changes
- [x] 4.2 Implement screen text extraction (trailing-space trim, wide-char continuation skip) and framed `screen` output; verify a test that the dump has at most `rows` lines no wider than `cols`
- [x] 4.3 Implement `expect-text` and the 0/1/2 exit statuses; verify tests for a passing expectation, a failing one (status 2, screen printed), and a script error (status 1)

## 5. State snapshot

- [x] 5.1 Implement the `AgentState` snapshot (version, seed, tick, player position and body, inventory, UI tab/modal, combat, capped nearby entities, capped recent messages) with sorted lists; verify a test on seed 12345 that the starting inventory and player position appear
- [x] 5.2 Add framed `state` output; verify a test that snapshotting the same state twice gives byte-identical JSON and that no tile data is present

## 6. CLI, recipe, docs

- [x] 6.1 Parse `--script`, `--seed`, `--size` in `main()` and run headless without touching the terminal; verify `cargo run -p lithicrivers-client -- --script <file>` works from a non-TTY shell (e.g. `| cat`)
- [x] 6.2 Replace the `client-blind` stub in `justfile` with `client-blind script:`; add `scripts/agent/smoke.txt` (skip splash, move, chop/mine, screen, state, expect-text); verify `just client-blind scripts/agent/smoke.txt` exits 0
- [x] 6.3 Add a test that runs the smoke script twice in-process and asserts identical output; verify it passes
- [x] 6.4 Update AGENTS.md: agents may run `just client-blind <script>` (not the interactive client), document script commands briefly, and note that gameplay time reads must use `app.clock`; verify the documented smoke command runs as written
- [x] 6.5 Add a Claude Code `PreToolUse` hook in `.claude/settings.json` that blocks Bash commands running `just client-linux`, `just client-windows`, or the client binary without `--script`, with a message pointing to `just client-blind <script>`; verify the hook blocks `just client-linux` and allows `just client-blind scripts/agent/smoke.txt`
- [x] 6.6 Add a project skill `.claude/skills/play-headless/SKILL.md` describing when to use headless scripts (testing, reproducing, checking a change in the running game) and the script syntax; verify the skill appears in a new Claude Code session's skill list
