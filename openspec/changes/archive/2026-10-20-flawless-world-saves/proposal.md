# Proposal

## Why

Saving and loading quietly loses game state. The save format (`SaveData`, version 4) stores a hand-picked subset of each entity type, so anything not on the list is dropped or reset on load:

- **Player:** `Body` is reset to default (damaged and missing body parts are healed); `Energy`, the combat `ActionQueue`, `BattleDelay`, `Stunned` and `Dead` are lost.
- **Sheep:** only position is saved; health resets.
- **Feral dogs:** inventory (their loot) and combat state are lost.
- **Corpses:** not saved at all, so unlooted corpses and their items vanish.
- **Resources:** `PlayerState` (noclip and fog-of-war toggles, combat-active flag, pending action, last-combat tick), the combat target tracker and the message log are not saved.
- **Client:** tutorial progress and hotbar assignments are not saved; loading mid-combat doesn't reopen the combat panel.

Each new component or system adds another thing to forget. Saves should be flawless: what you load is exactly what you saved, and nothing can be silently left out in the future.

## What Changes

- Save **every entity with every component** through one registry of saveable component types, preserving entity IDs so references between entities (combat targets, action queues) stay valid.
- A test fails if any entity in the game has a component type that isn't registered, so a new component can't be forgotten.
- Save all game resources that make up the world's state (tick, tiles, chunk/structure state, quests, player state, target tracker, message log). Only the global config (read from the config file) and the system scheduler (rebuilt at startup) are not saved.
- Save the client state that affects play: tutorial progress, hotbar assignments, the view position; loading mid-combat reopens the combat panel.
- A new headless step, `save-reload`, saves the running game and loads it back in place. The core acceptance test: a script with `save-reload` inserted anywhere produces exactly the same output as the same script without it.
- Save format version 5. Version-4 saves still load through the existing loader, with its known gaps.

## Capabilities

### New Capabilities
- `save-data`: What a save contains and the guarantee that loading a save restores exactly the saved state, including future behavior.

### Modified Capabilities
- `headless-client`: adds the `save-reload` script step (and MCP equivalent through `run_script`).

## Impact

- Core: `save_load.rs` rewritten around a component registry and `hecs` row serialization (enable the `row-serialize` feature); `Serialize`/`Deserialize` added to components and resources that lack them (`ActionQueue`, `QueuedAction`, `CombatAction`, `Move`, `PlayerState`, `PlayerIntent`, `TargetTracker`, `GameEvents`/`MessageLog`, ...).
- Client: a client-state section in the save (tutorial progress, hotbar), restore of combat UI on load; existing Menu-tab save/load keeps its keys.
- Headless: `save-reload` step, round-trip tests over several scenarios.
- Works with `add-title-and-world-menu` in either order: that change stores `world.json` using whatever `SaveData` is current.
