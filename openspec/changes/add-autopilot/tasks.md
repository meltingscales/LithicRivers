# Tasks

## 1. Foundations

- [ ] 1.1 Move crafting into `Game::craft(recipe_index)` and have the Crafting tab call it; verify the existing crafting tests and snapshots pass unchanged and a core test crafts a chest
- [ ] 1.2 The `autopilot` module: goal and step types, the goal stack, and the core driver applying steps through `Game`; verify `cargo test -p lithicrivers-core` for "get 3 planks" with logs in hand

## 2. Planning

- [ ] 2.1 "Get item" expansion over recipes, the mining table and creature loot, failing with the missing item; verify core tests for a chest from an empty inventory (TestWorld with trees and scrap) and for an item with no source
- [ ] 2.2 Long-range pathfinding (chunk graph, then tiles, digging and layer changes) in per-tick node budgets; verify core tests around walls, down a layer, and that one tick never exceeds the budget (`just util-bench` with a bot scenario in `scripts/bench/`)
- [ ] 2.3 Honest resource finding: explored tiles first, then a fixed spiral search by worldgen band; verify a test that the bot never paths to an unexplored ore it couldn't know about, and two runs give the same plan
- [ ] 2.4 Hunting and self-defense with Melee and Escape; verify a TestWorld test getting leather from a feral dog, and escaping when the torso is badly damaged
- [ ] 2.5 Build goals from blueprints and structure plans, and quest objectives from `QuestObjectiveType`; verify tests building a small blueprint and reaching a location objective

## 3. Client

- [ ] 3.1 The input driver turning steps into keybind presses through `headless::Runner`, recording them as a script; verify "get 1 chest" by the input driver, exported and replayed with `just client-blind` to the same state
- [ ] 3.2 `bot` cheat commands (gated on cheats), the MCP tool, the path overlay and Modes-panel status, and stopping on any player key; verify headless tests for each command, the refusal without cheats, and a snapshot of a running bot
- [ ] 3.3 Document the autopilot in `AGENTS.md` (commands, the script step, MCP) and add a `docs/AUTOPILOT.md` guide for speedrunners; verify by reading the diff

## 4. Completability

- [ ] 4.1 The completability test: from a seed-12345 world, the input driver finishes "A New Arm" and gets a chest, a burner generator, an electric furnace and a crude resurrection vat, each within a tick budget; verify `cargo test --workspace` (or the CI job if it is split out), and that removing iron scrap from the mining table makes it fail naming the item
- [ ] 4.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.3 User check: watch the bot play in the real client (`bot quest`, `bot get chest`) and judge whether it looks sensible and is useful for speedruns
