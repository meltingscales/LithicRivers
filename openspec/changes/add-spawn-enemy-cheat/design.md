# Design

## Context

- Cheats live in `crates/client/src/app/input/cheat_console.rs`: a `CommandRegistry` of `fn(&mut App, &[&str]) -> Result<(), Box<dyn Error>>` handlers. The console's Enter key calls `execute_command(app, &command)?`, so an `Err` would propagate out of `handle_input` and end `run_app` — the interactive client would quit. Existing handlers avoid this by logging usage errors and returning `Ok`, which also means headless `cheat` steps can't tell a bad command from a good one (`tp` with bad arguments silently does nothing).
- Headless `cheat` steps and the MCP `cheat` tool call `registry.execute` and already fail on `Err` (spec `headless-client`, "Cheat steps").
- Enemy spawning exists in core: `spawn_utils::world_spawn_feraldog(world, x, y, z, &GameWorld)`, inventory seeded from the world seed and position. Feral dogs are the only hostile kind.
- Movement blocking: tiles via `GameWorld::get_tile_cached(..).is_passable()`; entities via the `BlocksMovement` component (the player has it).
- Combat starts when a combat-capable enemy is adjacent to the player on a tick.

## Goals / Non-Goals

**Goals:**
- Self-contained combat setup for scripts, snapshots and MCP.
- One coordinate syntax for every position-taking cheat.
- Cheat failures visible everywhere, and never fatal.

**Non-Goals:**
- Spawning non-hostile creatures, NPCs or items (`spawn_enemy` only; a general `spawn` can follow the same pattern later).
- Retrofitting `giveitem`, `noclip_toggle` etc. to return errors; they keep their current log-and-continue behavior. Only `tp` changes, because it adopts the new coordinate parser.

## Decisions

### Enemy kinds and placement rules live in core
Add `spawn_utils::EnemyKind` (`FeralDog`, with `name()`, `from_name()`, `ALL`) and `spawn_utils::spawn_enemy(ecs_world, game_world, kind, Position) -> Result<Entity, SpawnError>`. It checks the tile is passable and no `BlocksMovement` entity is there, then calls the existing `world_spawn_feraldog`. Core owns spawning and the blocking rules, and this keeps the rule testable with `TestWorld` without the client.
Alternative: do the checks in the cheat handler. Rejected: it would duplicate movement-blocking logic in the client.

### `~` coordinates parsed in the client
`parse_position(args: &[&str], player: Position) -> Result<Position, String>` in `cheat_console.rs`: exactly three tokens, each `<i64>` or `~` / `~<i64>`. Both `tp` and `spawn_enemy` use it. Minecraft players already know `~`; it keeps absolute coordinates (used by existing scripts) working unchanged.

### Default position: two tiles east
`spawn_enemy <kind>` with no coordinates means `~2 ~ ~`: near the player and never on the player's tile. It does not delay combat: feral dog AI chases the player, so the dog is adjacent after the next tick and combat starts (observed while implementing; the original "not adjacent, so no combat yet" assumption was wrong). Scripts that want a quiet spawn can place the enemy further away. Exactly zero or three coordinates are accepted.

### One helper for tick results in the client
`App::apply_tick_result(result)`: on `CombatTriggered`, switch to the World tab and open the combat UI; on `CombatEnded`, close it. Every client tick site uses it: movement, interact, mining, building, combat actions, the combat auto-tick in `step`, and the headless `tick` step. Before, each site handled the result differently (only movement did both parts), so combat could start with no combat panel after mining, building, interacting from another tab, or a headless `tick`. This is the same bug class fixed for movement in #53; centralizing it fixes the root cause rather than one more call site.

### Errors: return `Err`, console catches it
Cheat handlers that can fail (`spawn_enemy`, `tp`) return `Err(message)`. The interactive console's `execute_command` catches the error and logs it with `log_red` instead of propagating with `?`. Headless and MCP keep their existing error path. This removes the latent "an Err closes the game" trap for all commands, current and future.

### Scripts stop depending on starting dogs
`combat-on-tutorial-tab.txt`, `combat-panel-stays-after-walking-away.txt` and `snapshots/combat-panel.txt` become `spawn_enemy feral_dog` + a step east (and `tp ~25 ~25 ~` to walk away). The bug scripts still fail on the pre-fix behavior by construction (same flow, different dog), and the combat-panel snapshot is re-recorded; its diff is only the dog's position, inventory and the surrounding map.

## Risks / Trade-offs

- [Spawned dog's inventory differs from the starting dog] → Expected; the snapshot change is reviewed and explained in the PR.
- [`~` conflicts with a shell or key binding] → Scripts and MCP pass text directly; in the console `~` is an ordinary character. No conflict.
- [Spawning far away generates new chunks] → `get_tile_cached` generates on demand, as `tp` already does; acceptable for a cheat.
