# AI Instructions

You are allowed to run `just build-linux` (or `just build-windows`), `just install`, and the `just demo-*` / `just util-*` recipes. This game uses ratatui for the UI.

If you're running NixOS, make sure to run `nix-shell` to get the right environment.

`./docs/*.md` contains lots of useful documentation.
`.ai/*.md` contains some prompts I reuse from time to time.

You should use `rustup` to run `cargo` and other tools.

Please avoid running `cargo clean`. It takes a long time to rebuild rust modules.

Never run the interactive client (`just client-linux` / `just client-windows`) or any other TUI: your terminal can't display it. To see the running game, use the headless client (below). If something can only be judged by a human looking at the real TUI, ask me to run `just client-linux`.

## Demos

All code prefixed with `demo_*.rs` is not meant to be part of the main codebase. These are self-contained demos that should not be impacted by main codebase changes or refactors.

## Testing

Run tests with `cargo test --workspace` to validate game systems (`just test-linux` also works, but runs a full `build-linux` first). Lint with `just code-clippy`, which treats warnings as errors and must pass. Write simple unit tests for:
- Combat mechanics (damage, death, action queues)
- Game logic (movement, energy, cooldowns)
- Edge cases and bug fixes

Tests help ensure systems work correctly and prevent regressions during development.

For game-logic tests in `lithicrivers-core`, build a scenario with `test_support::TestWorld` instead of `Game::new` (which spawns sheep, dogs, NPCs and structures). A fixture starts with only the player at (0, 0, 0); you place tiles and entities, get handles back, then drive real ticks:

```rust
use crate::test_support::TestWorld;

let mut world = TestWorld::new(3)
    .fill((-3, -3, 0), (3, 3, 0), TileKind::Grass)
    .inventory(&[])
    .auto_pickup(true);
world.item(1, 0, 0, ItemKind::Log, 4);
let mut game = world.build();
game.act(PlayerAction::Move { dx: 1, dy: 0, dz: 0 });
game.tick();
assert_eq!(game.player_inventory(), vec![(ItemKind::Log, 4)]);
```

Add creatures with `world.sheep(..)`, `world.feral_dog(..)`, `world.npc(..)`, and look them up with `game.pos(handle)` / `game.health(handle)`. Other crates' tests can use it by enabling the core `test-support` feature in `[dev-dependencies]`. If `Game::new` changes on purpose, the `game_new_matches_golden_entities` test fails and prints the new entity list for `crates/core/src/testdata/game_new_seed_12345.txt`.

## Headless client (seeing the running game)

`just client-blind <script>` runs the full client with no terminal, no audio and a fake clock, and prints text screen dumps and JSON state dumps. Use it to check a change in the real game, or to reproduce a bug as a committed script under `scripts/`. Example: `just client-blind scripts/agent/smoke.txt`.

Scripts have one step per line; `#` starts a comment.

| Step | Effect |
|---|---|
| `seed <u64>`, `size <cols>x<rows>` | Setup; only before other steps. Defaults: config `DEFAULT_SEED`, 160x50 |
| `skip-splash` | Jump past the splash screens to the main UI |
| `key <category>.<ACTION> [n]` | Press the key bound to a keybind (e.g. `movement.MOVE_SOUTH`, `action.MINE`), n times |
| `rawkey <NAME>` | Press a key by config name (`ENTER`, `ESCAPE`, `TAB`, `F1`, `q`) |
| `type "<text>"` | Press one key per character |
| `wait <ms>` | Advance the fake clock in 50 ms frames (splash timers, combat auto-ticks) |
| `tick [n]` | Advance the game simulation n ticks |
| `cheat <command>` | Run a cheat console command (`tp x y z`, `giveitem <item> <qty>`, ...) |
| `screen [label]` | Print the screen as text |
| `state [label]` | Print a JSON summary: player, body, inventory, tab/modal, combat, nearby entities, recent messages |
| `expect-text "<text>"` | Fail (exit 2) unless the screen contains the text |
| `quit` | Stop successfully |

Exit status: 0 ok, 1 script error, 2 failed `expect-text`. The same script, seed and build always print identical output, so all time-dependent behavior and rendering must read time from `app.clock`, never `Instant::now()` or `SystemTime::now()`.

## Planning with OpenSpec

For anything larger than a small fix, use OpenSpec (`openspec/`):

- Check `openspec/changes/` first. If a change already covers the work, continue it instead of starting a new one.
- New work: `/opsx:propose` to write the proposal, specs, design and tasks; `/opsx:apply` to implement; `/opsx:archive` when done.
- `openspec/specs/` describes current behavior. Read the relevant spec before changing a system it covers.
- Project rules for plans live in `openspec/config.yaml`.

## Converting TileKind to the name of a tile

TODO: Update `define_tiles!` in tile_registry.rs to add a tile name.

## Keybind handling

Avoid hardcoding keys (i.e. `KeyCode::Char('v')`). Make sure to populate `default_config.rs` as well as use the `keybinds.matches` method i.e. `app.ui.keybinds.matches("ui", "CLOSE_HELP_MENU", &key)` if you want to check for a keypress. This makes our game actually key-mappable.

Also, if you need to convert a key into a printable version, please use code like `app.core.config_manager.get_printable_key_for_keybind("movement", "MOVE_NORTHWEST"),`

## Guiding principles

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".

- See `docs/INSPIRATION.md` for inspiration.

- This game renders using ASCII text.

- docs/TODO.md contains a big, unedited list of my long-term goals with this game. By no means do we need to implement all of them. I will likely cut or edit parts of the TODO. I just want to focus on the section called "MVP for steam release (2026)".

Periodic tasks:

- Prune our various docs/ documents and make sure tasks are marked as completed, specifically `docs/STEAM-MVP.md`.

- Make sure rust warnings get handled.

- Examine code for any TODOs and make sure they are still relevant.