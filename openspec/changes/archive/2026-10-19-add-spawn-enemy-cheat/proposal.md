# Proposal

## Why

Scripts, snapshot tests and MCP sessions that need a fight currently teleport next to one of the feral dogs `Game::new` happens to place at (-15, -15, 0). That ties combat repros to starting-world details that will change, and gives no way to set up a fight anywhere else. A cheat that spawns an enemy where you want it, relative to the player, makes combat scenarios self-contained.

## What Changes

- New cheat command `spawn_enemy <kind> [x y z]` that spawns an enemy using the game's normal spawn code. Kinds: `feral_dog` (the only hostile creature today).
- Coordinates accept player-relative offsets with `~` (`~2 ~ ~` is two tiles east of the player); plain numbers stay absolute, per axis. Without coordinates the enemy appears two tiles east of the player; like any feral dog it then closes in, so combat starts on the next turn.
- `tp` accepts the same coordinate syntax (`tp ~5 ~ ~`), since it is the other cheat that takes a position.
- Spawning onto a tile that blocks movement or holds another blocking entity is refused with a message.
- Cheat command failures are reported instead of being silently ignored: the in-game console logs them in red, and headless scripts / MCP `cheat` calls fail with the message. Previously a cheat returning an error would have propagated out of the input loop and closed the interactive client.
- Every client path that advances the game (movement, interact, mining, building, combat actions, combat auto-ticks, and the headless `tick` step) applies the tick's combat transitions through one helper. Previously only movement opened the combat panel on the World tab; interact opened it without switching tabs, mining and building ignored combat starting, and the headless `tick` step entered combat in core with no combat UI. Found while testing `spawn_enemy`: a spawned dog walks up to the player on the next tick.
- The two combat bug scripts and the combat-panel snapshot spawn their own dog instead of relying on the starting dogs. Docs (AGENTS.md, `play-headless` skill, MCP `cheat` tool description) list the new command and syntax.

## Capabilities

### New Capabilities
- `cheat-console`: Developer cheat commands available from the in-game console, headless scripts and MCP: what they do, their coordinate syntax, and how failures are reported.

### Modified Capabilities
- `headless-client`: the `tick` step applies combat transitions to the client (combat panel opens/closes) like a player action, instead of only advancing core.

## Impact

- `crates/client/src/app/input/cheat_console.rs`: new command, shared coordinate parser, `tp` uses it, console logs command errors instead of propagating them.
- `crates/client/src/app/input/{world_movement,world_action,world_build,combat}.rs`, `main.rs` (`step`), `headless.rs` (`tick`): use one `App` helper for tick results.
- `crates/core/src/spawn_utils.rs` (existing `world_spawn_feraldog`) reused; a small placement check for free tiles.
- `scripts/agent/bugs/*.txt`, `scripts/snapshots/combat-panel.txt` and its snapshot (re-recorded with reason).
- `AGENTS.md`, `.claude/skills/play-headless/SKILL.md`, `crates/client/src/mcp.rs` tool description.
