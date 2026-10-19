# Proposal

## Why

AI agents (and CI) cannot exercise the game end-to-end: the client only runs as an interactive TUI, AGENTS.md forbids agents from launching it, and its loop is driven by wall-clock time, so even a captured run would not be reproducible. `just client-blind` already exists as a stub reserved for this purpose.

## What Changes

- Implement `just client-blind <script>`: runs the real client with no terminal, executing a line-based script of steps (keybind presses, raw keys, typed text, game ticks, fake-time waits, cheat commands, screen/state dumps, text expectations) and exiting with a status code.
- Screen dumps render the full client UI at a fixed size and print it as plain text.
- State dumps print a structured JSON summary of the game (player, inventory, nearby entities, combat, recent messages, tick, seed) with stable key ordering.
- The client loop advances on an injectable clock instead of reading wall-clock time directly, so the same script and seed always produce the same output. The interactive client keeps its current behavior.
- Headless runs never open an audio device or touch the terminal.
- AGENTS.md documents `just client-blind` as the agent-approved way to observe the running client.
- A Claude Code hook blocks agents from launching the interactive client and points them to `just client-blind`; a project skill teaches the script syntax when an agent needs to test in-game.

## Capabilities

### New Capabilities
- `headless-client`: Script-driven, terminal-free execution of the full game client, including script syntax, screen dumps, expectations, exit codes, and determinism guarantees.
- `game-state-snapshot`: A structured, deterministic JSON summary of the current game state for agents and tooling to read.

### Modified Capabilities
<!-- None: no existing specs. -->

## Impact

- `crates/client/src/main.rs`: `run_app` loop body extracted into a per-frame step usable by both the interactive loop and the script runner; CLI flag parsing in `main`.
- Client time sources (splash `on_tick`, combat auto-tick, dialogue text effects, hotbar double-tap) move to an app-owned clock.
- `crates/client/src/audio.rs` / `app_initialization.rs`: audio can be constructed disabled.
- New client module for the script runner and state snapshot; no new crate dependencies.
- `justfile` `client-blind` recipe; `AGENTS.md` testing section; `.claude/settings.json` hook; `.claude/skills/play-headless/`.
- Foundation for `add-tui-snapshot-tests` and `add-local-mcp-server`.
