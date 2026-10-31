# Proposal

## Why

The user wants a LithicRivers take on Minecraft's Baritone mod: an agent that pathfinds and pursues goals on its own, completing quest objectives, gathering resources, crafting items and building structures. It serves two purposes:

- **Speedrunning:** a tool-assisted run can hand whole goals ("get 4 iron scrap", "finish A New Arm") to the agent instead of pressing every key.
- **Completability tests:** CI proves the game can be finished from a fresh world, through the same inputs a human uses, so a recipe, worldgen or quest change that makes progress impossible fails a test instead of a playtest.

Today nothing checks this. The recipe-reachability test only proves an item has a recipe chain from raw materials, not that a player can actually walk to, mine, carry and craft them in a real world.

## What Changes

- **Goals:** go to a position, get N of an item, craft an item, place blocks from a blueprint or structure plan, and complete the active quest's next objective. Goals nest: "get a chest" plans "get planks" and "get nails", which plan mining logs and iron scrap.
- **A planner** that turns a goal into steps (walk to, mine, pick up, craft, place, interact, fight) from the game state, using the recipe tree, the mining drop table and long-range pathfinding across chunks and layers (stairs, vertical conveyors, digging down).
- **Two drivers for the same plans:**
  - **Core driver:** runs steps straight through the game's actions, fast, for core tests.
  - **Input driver:** runs steps as the key presses a player would make, through the headless client, so a finished run is a replayable script (and a TAS movie once `add-tas-mode` lands). This is the proof a human could do it.
- **Where players reach it:** `bot` commands in the cheat console (`bot goto`, `bot get`, `bot craft`, `bot build`, `bot quest`, `bot stop`), a `bot` headless script step, and an MCP tool. In the game it is offered only in worlds that allow cheats or are TAS worlds, so normal play and leaderboards stay unassisted.
- **Visible while running:** the planned path drawn on the world view and the current goal and step in the Modes panel. Any player key stops it.
- **Completability tests:** from a fresh seed-12345 world, the input driver finishes "A New Arm" within a tick budget, and obtains a set of key items (chest, burner generator, electric furnace, a crude resurrection vat). New quests and milestones add their goals to this test as they ship.
- **Deterministic:** plans depend only on game state, seed and tick, never the wall clock, so a bot run replays exactly.

- **Hunting for loot:** items only creatures carry (leather for string, nuclear waste) are got by finding the creature and fighting it with Melee until it drops them. Otherwise the bot steers clear of creatures, and if it is attacked it fights back with Melee or escapes when badly hurt.

Out of scope: smart combat (move choice beyond Melee and Escape), factory design (it builds given blueprints, it doesn't design them), and defining the game's final win condition.

## Capabilities

### New Capabilities
- `autopilot`: goals, planning, the core and input drivers, the `bot` commands, and the completability tests.

### Modified Capabilities
<!-- None: crafting moves into core (see design.md) without changing what the player sees. -->

## Impact

- **Core:** a new `autopilot` module (goals, planner, long-range pathfinder, core driver). Crafting moves from the client's input handler into `Game::craft(recipe)` so both drivers and the player share it.
- **Client:** the input driver (plan steps to key presses through `headless::Runner`), the `bot` cheat command and script step, the MCP tool, the path overlay and Modes-panel status.
- **Tests:** completability tests in the client crate; they are slow, so they get a tick budget and run in CI like the snapshot tests.
- **Depends on / works with:** `add-tas-mode` (movies), `blueprints` (build goals), `arm-replacement-quest`.
