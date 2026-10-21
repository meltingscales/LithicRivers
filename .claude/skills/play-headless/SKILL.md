---
name: play-headless
description: Run, test, or inspect the LithicRivers game client without a terminal using headless scripts (just client-blind), and maintain the UI snapshot tests in scripts/snapshots. Use when checking a change in the running game, reproducing or demonstrating a bug, looking at what a screen or panel shows, verifying game state after actions, or when a snapshot test fails or a UI change needs new snapshots. Never launch the interactive client.
---

# Play the game headless

The interactive client (`just client-linux`) needs a real terminal; agents must not run it. Instead, write a script and run:

```
just client-blind path/to/script.txt
```

It runs the real client with no terminal, no audio, and a fake clock. Output is deterministic: the same script, seed and build always print the same bytes.

## Workflow

1. Write a script. Put throwaway scripts in your scratchpad; put bug reproductions and checks worth keeping in `scripts/agent/`.
2. Start with `seed`, `size`, then `skip-splash`. To test the title menu instead, use `rawkey ENTER` twice to reach the title screen and drive it with `title.UP` / `title.DOWN` / `title.SELECT` / `title.BACK` (see `scripts/snapshots/world-list-one-world.txt`). Saves go to a temporary folder per run.
3. Get to the situation you need with `cheat tp x y z`, `cheat spawn_enemy feral_dog [x y z]`, `cheat giveitem <item> [qty]`, `cheat givecraftingcomponents <item> [amount] [raw]`, keybind presses, and `tick`/`wait`. Coordinates take `~`/`~<offset>` for player-relative axes (`cheat tp ~5 ~ ~`). For a fight: `cheat spawn_enemy feral_dog` then `key movement.WAIT`.
4. Observe with `state` (JSON: player, body, inventory, tab/modal, combat, nearby entities, recent messages) and `screen` (text render). Prefer `state` for facts; use `screen` for layout.
5. Assert with `expect-text "..."` so the script fails (exit 2) if the behavior regresses.

## Steps

| Step | Effect |
|---|---|
| `seed <u64>`, `size <cols>x<rows>` | Setup; only before other steps |
| `skip-splash` | Start a new world (run seed, cheats on) and go straight to the main UI, skipping splashes and the title menu |
| `key <category>.<ACTION> [n]` | Press the key bound to a keybind, n times |
| `rawkey <NAME>` | Press a key by config name: `ENTER`, `ESCAPE`, `TAB`, `LEFT`, `F1`, `q` |
| `type "<text>"` | One key press per character |
| `wait <ms>` | Advance fake time in 50 ms frames |
| `tick [n]` | Advance the simulation n ticks |
| `cheat <command>` | Cheat console: `tp`, `spawn_enemy`, `giveitem`, `givecraftingcomponents` (ingredients for a recipe; add `raw` for raw materials), `noclip_toggle`, `toggle_fogofwar`, `toggle_tutorial`; a failing cheat fails the script |
| `screen [label]` / `state [label]` | Print a framed text screen / JSON state |
| `expect-text "<text>"` | Exit 2 unless the screen contains the text |
| `save-reload` | Save and load back in place; output must not change, so use it to check that a state survives saving |
| `quit` | Stop successfully |

Keybind names come from `crates/core/src/default_config.rs` (categories like `movement`, `action`, `build`, `ui`, `inventory`). Exit status: 0 ok, 1 script error (bad syntax, unknown keybind or cheat), 2 failed expectation.

## Example

`scripts/agent/smoke.txt`:

```
seed 12345
size 120x40
skip-splash
key movement.MOVE_SOUTH 2
state walked
cheat tp 0 0 0
key action.MINE
key build.PLACE_EAST
expect-text "You chop the tree"
state chopped
```

`--seed` and `--size` on the command line override the script (`cargo run -q -p lithicrivers-client --bin lithicrivers-client -- --script s.txt --seed 7`).

## Exploring live over MCP

If the `lithicrivers` MCP tools are available (from the repo's `.mcp.json`), they run the same steps interactively: `new_game`, `press_key`, `cheat`, `wait`, `tick`, `screen`, `state`, `run_script`, and so on. They're good for poking around. When you find something, call `export_script`, save it under `scripts/agent/bugs/`, and replay it with `just client-blind` before reporting it.

## Snapshot tests

`cargo test` runs every `scripts/snapshots/*.txt` and compares its output with `crates/client/src/snapshots/*.snap`.

- New screen to guard: add a script there (`seed`, `size 100x30`, `skip-splash`, an `expect-text`, then `screen`/`state`), run `cargo test -p lithicrivers-client`, and accept the pending snapshot with `cargo insta accept` after reading it.
- A snapshot failed: read the diff. Regression: fix the code. Intended change: `cargo insta review` / `cargo insta accept`, or `INSTA_UPDATE=always cargo test -p lithicrivers-client`. Commit the `.snap` changes with the code change and explain them in the PR.
- Reproduced a bug: keep the script in `scripts/agent/bugs/` and add a test in `crates/client/src/headless.rs` that runs it (see the existing bug tests there).

