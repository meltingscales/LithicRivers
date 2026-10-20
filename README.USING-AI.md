# Using AI agents on LithicRivers

A short guide to building features with an AI coding agent (Claude Code, and to a lesser extent Codex, Cursor, Copilot) in this repo. For building and running the game yourself, see [QUICKSTART.md](QUICKSTART.md). Back to the [README](README.md).

## What's already set up

| Piece | Where | What it does |
|---|---|---|
| Agent instructions | [`AGENTS.md`](AGENTS.md) (`CLAUDE.md` imports it) | Project rules every agent reads: determinism, keybinds, testing, headless client, OpenSpec |
| OpenSpec | [`openspec/`](openspec/) | Plans (`changes/`) and the current behavior contract (`specs/`) |
| `/opsx:*` commands | `.claude/commands/opsx/`, `.claude/skills/openspec-*` | Propose, apply, update and archive changes from Claude Code |
| Headless client | `just client-blind <script>` | Lets agents play the real game with no terminal and get text screens and JSON state |
| Snapshot tests | `scripts/snapshots/`, `crates/client/src/snapshots/` | `cargo test` fails if a screen changes unexpectedly |
| Local MCP server | `.mcp.json` → `just client-mcp` | `lithicrivers` tools for exploring the game live |
| `play-headless` skill | `.claude/skills/play-headless/` | Teaches agents the script syntax when they need to test in-game |
| Deny rules | `.claude/settings.json` | Stop agents from launching the interactive TUI, which they can't see |
| PR checks | `.github/workflows/pr-checks.yml` | Clippy + tests on Linux, Windows, macOS for every PR |

Agents never run `just client-linux` / `just client-windows`. When something needs a human eye on the real TUI, the plan says **user check** and the agent asks you.

## The loop: propose → review → apply → merge → archive

Everything bigger than a small fix goes through an OpenSpec change. In Claude Code:

1. **Propose.** Describe the feature in plain words:
   ```
   /opsx:propose add a spawn-enemy cheat command
   ```
   The agent reads the relevant code and writes `openspec/changes/<name>/`:
   - `proposal.md`: why and what
   - `specs/<capability>/spec.md`: the behavior contract as requirements with WHEN/THEN scenarios
   - `design.md`: how, with the alternatives it rejected
   - `tasks.md`: checkboxes, each with how it will be verified (a test, a headless script, a snapshot, or a user check)

   Nothing in the game code changes at this step.

2. **Review the plan.** This is the cheapest point to steer. Read the specs and the "decisions" and "non-goals" in the design. Add requests mid-way ("also allow `~` offsets from the player") and the agent folds them in, or edit the files yourself and run `/opsx:update <name>` to make the artifacts consistent again.

3. **Apply.**
   ```
   /opsx:apply add-spawn-enemy-cheat
   ```
   The agent works through `tasks.md`, ticking each box only when its check passes. It **pauses and asks** when the plan turns out wrong or the work needs more scope than planned. In this repo it found, for example, that feral dogs chase the player (so the spec's assumption was wrong) and that a headless step could start combat without showing it. You pick an option, and it updates the plan before continuing.

4. **PR and merge.** Ask the agent to commit and open a PR. The description lists what changed, how it was tested, any snapshot changes with reasons, and any user checks for you. PR checks must pass.

5. **Archive.**
   ```
   /opsx:archive add-spawn-enemy-cheat
   ```
   The change's spec deltas merge into `openspec/specs/` (the source of truth for current behavior) and the change moves to `openspec/changes/archive/`. Archive PRs touch only OpenSpec files, so CI skips them.

Check what's in flight any time with `openspec list` (changes) and `openspec list --specs` (capabilities).

## Writing a good feature request

- **Say what the player sees,** not how to code it: "a title screen with Play Game / Options / Quit, where Play Game lists saved worlds".
- **Name the edges you care about:** defaults, limits, what happens on bad input, what must not change.
- **Mention the inspiration** if there is one ("Minecraft-style", "like ULTRAKILL's style meter"); see [docs/INSPIRATION.md](docs/INSPIRATION.md).
- **Keep one feature per change.** Several related ideas? Run `/opsx:propose` once per idea so each gets its own plan, PR and archive.
- **Not sure yet?** `/opsx:explore <idea>` is a thinking-partner mode: it investigates and discusses without writing a plan.

## Bugs

- **Small, obvious bug:** just describe it ("if you're on the tutorial tab, the combat GUI doesn't show up"). The agent reproduces it first as a headless script in `scripts/agent/bugs/`, fixes it, confirms the script fails without the fix, and adds the script to `cargo test` so it can't come back.
- **Bug that needs design decisions:** use `/opsx:propose fix …` like a feature.

## Watching and testing the game without a terminal

- **Scripts** (reproducible, CI-safe):
  ```
  just client-blind scripts/agent/smoke.txt
  ```
  One step per line (`key movement.MOVE_SOUTH`, `cheat spawn_enemy feral_dog`, `screen`, `state`, `expect-text "..."`). Full table in [AGENTS.md](AGENTS.md#headless-client-seeing-the-running-game).
- **MCP** (live exploration): in Claude Code the `lithicrivers` tools let an agent play a session step by step, then `export_script` turns the session into a replayable script. Restart Claude Code after changes to the client so the tools pick them up.
- **Snapshots:** if a screen changes on purpose, accept it with `cargo insta review` and say why in the PR. Never accept snapshot changes blindly.

## Tips

- **Ask for honesty and verification.** "Give me your honest opinion of this code" and "prove it with a test or script" both work well here.
- **Review user checks.** The agent can't see the real TUI. Anything marked "user check" is yours.
- **Other agents:** Codex, Cursor, Copilot and others read `AGENTS.md` and can run `just client-blind` and `cargo test`. The `/opsx:*` slash commands are Claude Code features; elsewhere, use the `openspec` CLI (`openspec list`, `openspec status --change <name>`, `openspec instructions apply --change <name>`) and ask the agent to follow it.
- **Keep docs honest.** If an agent finds that `AGENTS.md` or a spec is wrong, have it fix the doc in the same PR.
