# Tasks

## 1. Core spawning

- [x] 1.1 Add `spawn_utils::EnemyKind` (`FeralDog`; `name`, `from_name`, `ALL`) and `spawn_enemy(ecs_world, game_world, kind, pos) -> Result<Entity, SpawnError>` that refuses impassable tiles and tiles with a `BlocksMovement` entity, then uses `world_spawn_feraldog`; verify `TestWorld` tests for a successful spawn (full health), an occupied tile (player's own tile), an impassable tile, and same-seed same-position determinism, all passing with `cargo test --workspace`

## 2. Cheat console

- [x] 2.1 Make the interactive console catch command errors in `execute_command` and log them with `log_red` instead of propagating; verify a client unit test that executing a failing command through the console path leaves the app running and adds a red message
- [x] 2.2 Add `parse_position(args, player)` supporting absolute and `~`/`~<offset>` per axis; verify unit tests for absolute, relative, mixed, bare `~`, malformed (`~x`), and wrong argument counts
- [x] 2.3 Switch `tp` to `parse_position` and return errors for bad input; verify a headless script `cheat tp ~5 ~ ~` moves the player by 5 and `cheat tp ~x 0 0` exits with status 1 (tests in `headless.rs`)
- [x] 2.4 Register `spawn_enemy <kind> [x y z]` (default `~2 ~ ~`) using `parse_position` and core `spawn_enemy`, logging the spawn position; verify headless tests: default position two tiles east with combat not yet active, the dog closing in and combat starting (panel shown) after one turn, relative position, unknown kind lists `feral_dog`, occupied tile fails with status 1
- [x] 2.5 Add `App::apply_tick_result` (CombatTriggered: World tab + combat UI; CombatEnded: close it) and use it at every client tick site, including the headless `tick` step; verify a headless test that a `tick` step during which a dog reaches the player shows the combat panel and reports combat active, and that the existing bug scripts and snapshots still pass

## 3. Scripts and docs

- [x] 3.1 Rewrite `scripts/agent/bugs/combat-on-tutorial-tab.txt` and `combat-panel-stays-after-walking-away.txt` to spawn their own dog (no `tp -16 -16 0`); verify both still pass via their `cargo test` wrappers, and that each fails when its fix is temporarily reverted
- [x] 3.2 Rewrite `scripts/snapshots/combat-panel.txt` to spawn its own dog and re-record its snapshot; verify `cargo test --workspace` passes twice with no pending snapshots, and record the reason for the diff for the PR
- [x] 3.3 Document `spawn_enemy` and `~` coordinates in AGENTS.md's step table, the `play-headless` skill, and the MCP `cheat` tool description; verify `just code-clippy` passes and the documented example `cheat spawn_enemy feral_dog ~2 ~ ~` runs in a `just client-blind` script
