# Design

## Context

- **Actions:** the player acts through `Game` methods the client calls (`queue_player_move`, `queue_mine_at`, `queue_pick_up_at`, `install_part_into`, factory `place`, interactions) and through the client's input handlers. Crafting is the exception: it lives in `app/input/crafting.rs` and edits the inventory directly.
- **Knowledge the planner can reuse:** `crafting::ingredient_tree`, `raw_sources_of`, `mining_drop`, `CREATURE_LOOT`, blueprints and structure plans (`structure_plan.rs`), quest objectives (`quests.rs`, `QuestObjectiveType`).
- **Pathfinding today:** `pathfinding::Pathfinder` is a creature A* capped at 300 nodes and 15 tiles: fine for a dog, useless for a 1,600-tile walk to the facility.
- **Determinism:** everything must come from state, seed and tick (`AGENTS.md`). The headless runner and fake clock make any input sequence replayable.
- **Budget:** 16 ms per tick and per frame (`performance-budget`).

## Goals / Non-Goals

**Goals:**
- One planner, two drivers, so core tests are fast and the input-driven tests prove a human could do the same.
- Plans that a reader can follow in the log ("get Chest: need 4 Nail: mine Scrap Metal at (14, -3, -2)").
- Failing loudly and specifically when a goal is impossible.

**Non-Goals:**
- Optimal speedrun routes. The planner is greedy and good enough; a speedrunner improves on it by hand.
- Machine learning or search over the whole game.
- The game's final win condition.

## Decisions

### Goal stack, backward chaining
A plan is a stack of goals, expanded top-down:
- **Get N item:** done if held; else if a recipe makes it, push "craft" after "get" for each ingredient (the cheapest recipe by raw-material count, ties by recipe order); else if a tile drops it, push "mine the nearest such tile"; else if a creature carries it, push "hunt the nearest such creature"; else fail naming the item.
- **Go to:** a path, then walking steps; re-planned when blocked.
- **Build:** for each block of the blueprint, in a fixed order (bottom layer first, then row by row), "get the item" then "place it here".
- **Quest objective:** maps `QuestObjectiveType` to the goals above (reach a location, have an item, install a part, talk to an NPC).

Each step is a small enum (`Walk(dir)`, `Mine(pos)`, `PickUp(pos)`, `Craft(recipe)`, `Place(item, pos, facing)`, `Interact(pos, choice)`, `Attack`, `Escape`). The planner re-expands from the current state after every step, so surprises (a dog, a full inventory) are handled by re-planning, not by special cases.

Alternative: a general search (GOAP over all actions). Rejected: much slower, harder to read, and the recipe tree already gives the order.

### Long-range pathfinding: chunk graph, then tiles
- A coarse A* over chunks (`CHUNK_SIZE` tiles each) on the target layer, using whether a chunk has a walkable crossing, picks the corridor.
- A fine A* over tiles inside the next two chunks follows it. Mining a block costs its mining time, so digging is chosen when it is shorter than walking around.
- Layer changes use known stairs and vertical conveyors, else dig down or build stairs from the inventory.
- Search runs in slices of a fixed node budget per tick, so planning never goes over the frame budget; the bot waits a tick while a path is pending.

Alternative: extend the creature `Pathfinder`. Rejected: its limits exist to keep creature AI cheap, and creatures shouldn't pay for the bot.

### Finding resources without cheating
The planner may only use what the player could know: explored tiles (fog of war) and the mining table. To find a tile it hasn't seen, it explores outward in a fixed spiral from the player, layer by layer down the worldgen bands where `mining_drop` says the item can occur. This keeps the bot honest for the "a human could do it" claim.

### Crafting moves into core
`Game::craft(recipe_index) -> Result<(), CraftError>` takes over the inventory logic from `app/input/crafting.rs`; the Crafting tab calls it. The core driver calls it directly. The input driver crafts the way a player does: Crafting tab, search for the recipe name (`search.SEARCH`, `type`, `search.END_SEARCH`), activate.

### Input driver
In the client crate, a translator turns each step into keybind presses for the current UI state (World tab movement keys, `action.MINE`, the hotbar picker for placing, `action.INTERACT` and the interaction grid, the combat menu). It feeds them to `headless::Runner` one at a time and records them, so the run exports as a script (and a TAS movie later). It re-reads the game state after every key, never the screen text.

### Player access and gating
- Cheat console: `bot goto x y z`, `bot get <item> [n]`, `bot craft <item> [n]`, `bot build <blueprint>`, `bot quest`, `bot status`, `bot stop`. Headless: the `cheat bot ...` step works as any cheat. MCP: a `bot` tool with the same arguments.
- Allowed when `world_settings.cheats_allowed` (and later `tas_mode`). In the game it uses the core driver (it can act on the live world each tick); tests use the input driver.
- The path overlay draws on the world view in a dim color; the Modes panel shows `BOT get Chest: mining (14, -3, -2)`.

### Completability tests
`autopilot_completes_the_milestones` (client crate) creates a seed-12345 world with `skip-splash`, then runs each milestone with the input driver and a tick budget, asserting it finishes and printing the plan on failure. Milestones are a list in the test, so a new quest adds one line. If it gets slow, it can move behind a `just test-completable` recipe that CI runs, like the snapshot tests.

## Risks / Trade-offs

- [The input driver depends on UI details (menu order, key names) and breaks on UI changes] → It reads keybinds from config, never hardcodes keys, and finds menu rows by name; a UI change that breaks it is exactly what the test should catch.
- [Long runs make the test suite slow] → Tick budgets per milestone, no screen rendering between keys, and the option to run it as its own CI job.
- [Greedy plans get stuck in odd terrain] → Re-planning after each step, a stuck detector (no progress in N steps) that fails with the position, and worldgen pinned by seed so failures reproduce.
- [Exploring for unseen resources is slow] → Spiral exploration is bounded; the failure names the item and the area searched.

## Open Questions

- Whether a future TAS leaderboard should allow bot-assisted runs as their own category. It doesn't change this design.
