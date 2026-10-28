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

In Claude Code, permission deny rules in `.claude/settings.json` block Bash commands that would launch the interactive client: the `client-linux` / `client-windows` / `client-release-*` recipes, and any direct `cargo run` or `target/*/lithicrivers-client` launch (even with `--script`; use `just client-blind` / `just client-mcp` instead). A blocked command fails with a generic "Permission ... denied" message. Mentioning the recipe in a heredoc, a quoted string, a commit message or a `grep` is fine.

Scripts have one step per line; `#` starts a comment.

| Step | Effect |
|---|---|
| `seed <u64>`, `size <cols>x<rows>` | Setup; only before other steps. Defaults: config `DEFAULT_SEED`, 160x50 |
| `skip-splash` | Skip the splash screens and title menu: start a new world with the run's seed and cheats on, and show the main UI |
| `world <type>` | Like `skip-splash`, but a debug world type: `normal`, `void`, `ore-test`, `dog-arena` or `factory-showcase`. Nothing is saved |
| `click <col> <row> [left\|right\|middle] [shift]`, `scroll <up\|down> [col row] [shift]`, `move <col> <row>` | Mouse events at a screen cell, through the `mouse` bindings: on the World tab left click opens what's there, else places the hotbar block, else looks; right click breaks (both up to 5 tiles from the player); middle click picks a block; scroll cycles hotbar slots and Shift + scroll rotates; elsewhere scroll moves lists, and clicking the menu bar switches tabs. At 100x30 the player's cell is (38, 10), or (38, 9) in Place mode |
| `beat <ms>` | Demo pacing: in a demo scenario, steps before the first `beat` set the scene at once and later steps play one per beat. A no-op in scripts |
| `key <category>.<ACTION> [n]` | Press the key bound to a keybind (e.g. `movement.MOVE_SOUTH`, `action.MINE`), n times |
| `rawkey <NAME>` | Press a key by config name (`ENTER`, `ESCAPE`, `TAB`, `F1`, `q`) |
| `type "<text>"` | Press one key per character |
| `wait <ms>` | Advance the fake clock in 50 ms frames (splash timers, combat auto-ticks) |
| `tick [n]` | Advance the game simulation n ticks |
| `cheat <command>` | Run a cheat console command: `tp x y z`, `tp_quest [android|facility|locker]` (beside a quest spot; with no target, the arm quest's next one), `spawn_enemy <kind> [x y z]` (kinds: `feral_dog`; default two tiles east), `giveitem <item> [qty]` (qty defaults to 1), `givecraftingcomponents <item> [amount] [raw]` (what crafting the item needs, or with `raw` its raw materials), `noclip_toggle`, `toggle_fogofwar`, `toggle_tutorial`, `set_time hh:mm` (skips forward to the next time the clock reads that: a day is 15 game minutes, tick 0 is 08:00, and the surface is dark from 21:00 to 05:00; `scripts/snapshots/day-night-cycle.txt`), `keepinventory [on|off]` and `keepbody [on|off]` (dying keeps the inventory; respawning restores the old body, repaired, missing parts still missing; saved with the world), `kill_player` or `suicide` (destroys the player's body: the system-failure screen offers Respawn at the bunker, `key ui.MENU_ACTIVATE`, or Save and quit, after `key movement.MOVE_SOUTH`). Coordinates can be relative to the player: `~` or `~<offset>` per axis, e.g. `spawn_enemy feral_dog ~2 ~ ~` |
| `screen [label]` | Print the screen as text |
| `state [label]` | Print a JSON summary: player, body, inventory, tab/modal, combat, nearby entities, recent messages |
| `expect-text "<text>"` | Fail (exit 2) unless the screen contains the text |
| `save-reload` | Save the game (with tutorial progress, hotbar and view) in memory and load it back; fails (exit 1) if either step fails |
| `quit` | Stop successfully |

Without `skip-splash`, the client starts at the logo; two `rawkey ENTER` reach the title screen, which scripts drive with the `title.*` keybinds (`title.UP`, `title.DOWN`, `title.SELECT`, `title.BACK`, `title.LEFT`/`title.RIGHT` for toggles and volume, `title.DELETE_CHAR` in text fields; `type` fills the name and seed fields). Each headless run and MCP session gets its own empty saves folder, removed when it ends, so worlds created in a script never touch real saves and the world list always starts empty. Options changes are not written to the config file in headless runs.

Factory blocks: `cheat giveitem chest 1`, then `key build.TOGGLE_BREAK_PLACE_MODE 2` (Place mode), `key hotbar.HOTBAR_SLOT_1 2` (double tap opens the block picker), `key movement.MOVE_SOUTH 2` (the picker lists Clear slot, then placeable items in inventory order; with the starting kit that is Acorn, then Chest), `key ui.MENU_ACTIVATE` (assign the chest), and `key build.PLACE_<DIRECTION>`. `key build.ROTATE_PLACEMENT` turns the next placement; one more `TOGGLE_BREAK_PLACE_MODE` gives Rotate mode, where the place-direction keys turn a placed block. `key action.PASS_TIME` lets 10 game seconds (1000 ticks) pass. Wooden conveyors (`cheat giveitem wooden_conveyor 4`) are placed the same way; `key action.INTERACT` next to an empty belt offers "Put <item> on belt" for the item selected in the Inventory tab. Power: `cheat giveitem power_pole` / `burner_generator`; Interact on a generator opens its fuel window, on a pole its network info. See `scripts/snapshots/factory-chest-window.txt`, `factory-belt-line.txt`, `factory-generator-window.txt` and `factory-power-wires.txt` (wires blink each second of the fake clock, so `wait 1000` toggles them). Machines: `cheat giveitem auto_planter` / `auto_harvester`; Interact opens their window (planter acorns, harvester output). `scripts/snapshots/factory-tree-farm.txt` builds and runs the reference tree farm. Crude furnaces (`cheat giveitem crude_furnace`) open a three-slot window: Your Items has the focus with the Input slot selected, the side keys switch panels, and activating stores the selected item in the selected slot or takes one out (`scripts/snapshots/factory-furnace-window.txt`). Electric furnaces (`cheat giveitem electric_furnace`) have no fuel slot and run on grid power; the showcase's last example feeds one from a chest down long belts (`scripts/snapshots/factory-electric-furnace-window.txt`). Logistics: `cheat giveitem extractor` / `filter_extractor` / `belt_router`; Interact on a filter extractor or router opens its filter picker (row 0 is no filter, then items in `ItemKind` order: Log, Wooden Plank, Acorn, ...; `scripts/snapshots/factory-filter-picker.txt`). Belts are walkable and carry the player a tile every factory step (about 1.7 tiles per game second). Factory rates are in factory seconds, 3 game seconds each (`FACTORY_SECOND_TICKS`). Vertical conveyors (`cheat giveitem vertical_conveyor`) hand items to the block directly above, or below after a Rotate; the showcase's last example lifts logs to z = 1 and back (`scripts/snapshots/factory-vertical-conveyor.txt`). The player climbs vertical conveyors like stairs. Stairs (`cheat giveitem stairs 2`) placed from the hotbar build a two-tile stairway up (the rotate key switches to down) and cost 2; see `scripts/snapshots/stairs-ghost.txt`.

Blueprints: `key build.BLUEPRINT_MODE` enters Blueprint mode (copying while the library is empty). The place-direction keys move a cursor that starts one tile east of the player, `key build.PLACE_CENTER` marks a corner (twice to save a blueprint) or pastes with the cursor as the top-left corner, `key build.ROTATE_PLACEMENT` turns the paste, and pasting over player-built blocks needs the same paste twice. `key build.BLUEPRINT_LIBRARY` opens the Blueprints window. See `scripts/snapshots/blueprint-paste-preview.txt`.

Item Dictionary: the Items tab sits after Crafting (from `skip-splash`, which starts on the Tutorial tab, `key ui.MENU_NEXT 6`). `key search.SEARCH` starts a search, `type` fills it and `key search.END_SEARCH` keeps the filter (`scripts/snapshots/item-dictionary.txt`). The Crafting tab (`key ui.MENU_NEXT 5`) has the same search over recipe, result and ingredient names (`scripts/snapshots/crafting-search.txt`). In the cheat console, `key ui.CHEAT_CONSOLE_SCROLL_DOWN` picks a suggestion and `rawkey ENTER` fills it in; a second `rawkey ENTER` runs it.

Demo mode: Watch Demo on the title screen plays the scenario scripts in `crates/client/assets/demos/` (listed in order in `app::demo::SCENARIOS`) under a scrolling DEMO banner; Escape returns to the title. Scenarios are ordinary scripts, so `app::demo::tests::every_scenario_runs_as_a_script` runs each one and snapshots its last frame: when a game change breaks a scenario, fix the script (and use `world <type>` rather than Create New World, which would save).

Debug worlds: a script creates one from the title screen. With Developer mode on (the default), World type is the row after Developer mode, and `title.RIGHT` steps through Normal, Void, Ore test, Dog arena and Factory showcase: two `rawkey ENTER`, `key title.SELECT 2`, `key title.DOWN 4`, `key title.RIGHT <n>`, `key title.DOWN 2`, `key title.SELECT`, then two `rawkey ENTER` to skip the boot intro. See `scripts/snapshots/debug-world-showcase.txt`. In Rust tests, use `Game::new_with_settings(seed, settings)` with `settings.world_type` set.

When Interact finds more than one choice, the Choose Interaction menu shows a 3x3 grid of the tiles around the player: a movement key (`key movement.MOVE_WEST`, or `key movement.WAIT` for the player's own tile) runs that tile's choice, pressing it again moves through a tile's several choices, and `key ui.MENU_UP` / `key ui.MENU_DOWN` walk the whole list (`scripts/snapshots/interaction-grid-two-on-a-tile.txt`).

To set up a fight, spawn the enemy instead of relying on where the starting world places creatures: `cheat spawn_enemy feral_dog` then `key movement.WAIT` (feral dogs chase the player, so combat starts on that turn).

In Rust tests, build a run from a list of steps with `headless::chain_commands(&["skip-splash", "cheat tp 0 0 0"])` (seed 12345, 100x30) and continue it with `runner.chain(&["key movement.WAIT"])`; both fail the test with the step's error.

Exit status: 0 ok, 1 script error (including a cheat that fails), 2 failed `expect-text`. The same script, seed and build always print identical output, so all time-dependent behavior and rendering must read time from `app.clock`, never `Instant::now()` or `SystemTime::now()`.

### Snapshot tests (catching UI regressions)

Every script in `scripts/snapshots/*.txt` runs during `cargo test`, and its full output is compared with a stored snapshot in `crates/client/src/snapshots/`. Build-specific text (version, git branch/commit, config file path) is masked.

- **Add coverage:** write a script in `scripts/snapshots/` (start with `seed`, `size 100x30`, `skip-splash`; add an `expect-text` so the script itself fails loudly if it stops reaching the right screen), then accept its new snapshot.
- **A snapshot test fails:** read the diff in the test output. If the change is a regression, fix the code. If it is intended, accept it with `cargo insta review` (interactive; `just install` installs `cargo-insta`) or `cargo insta accept`, or re-record everything with `INSTA_UPDATE=always cargo test -p lithicrivers-client`.
- **Never accept snapshots blindly.** Commit updated `.snap` files together with the change that caused them, and say in the PR why each snapshot changed.
- Bug reproductions belong in `scripts/agent/bugs/` with a test that runs them; screens worth guarding belong in `scripts/snapshots/`.

### Live exploration over MCP

For open-ended exploration ("play the crafting flow and look for bugs"), the repo's `.mcp.json` registers a local MCP server, `lithicrivers` (`just client-mcp`; stdio only, no network, no auth). Its tools mirror the script steps: `new_game`, `press_key`, `raw_key`, `type_text`, `wait`, `tick`, `cheat`, `screen`, `state`, `expect_text`, `run_script`, and `export_script`. It uses the same fake clock, so the same calls give the same results.

- Use **scripts** (`just client-blind`) for anything that must be reproducible or checked in CI; use **MCP** to explore.
- A bug found over MCP is not reproduced until you call `export_script`, save the result under `scripts/agent/bugs/`, and confirm `just client-blind` shows the bug. Add a test that runs it, as the existing bug scripts do.
- The first launch builds the client, which can be slow; run `cargo build -p lithicrivers-client` first if the MCP connection times out.

## Saves

A save holds every entity with every component, all of `Resources` except the config, and the client's play state (view, tutorial progress, hotbar). To keep it complete:

- Register every new component type in `saved_components!` in `crates/core/src/save_load/mod.rs`, with a stable key. `every_component_is_registered` fails if an entity carries an unregistered type; add the spawn path you created to `worlds_from_every_spawn_path` there.
- New `Resources` fields are saved automatically (it derives serde); mark state that is rebuilt on load `#[serde(skip)]`.
- Client state that changes what happens next goes in `ClientSave` (`crates/client/src/app/save.rs`).
- `save_reload_anywhere_changes_nothing` runs every committed script with `save-reload` inserted after each step and fails if any output changes.

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

- See `docs/INSPIRATION.md` for inspiration. For factory, crafting and tech-tree work the main references are Minecraft (with GregTech, Tekkit, Big Dig and Create), Factorio and Caves of Qud; for combat and feel, ULTRAKILL, CICADAMATA and Downwell. Borrow mechanics, not content: blocks, materials and lore should be unique to LithicRivers.

- Multiblock structures (the bunker, the facility, ships, temples) live in `crates/client/assets/structures/`. Write new ones as a `plan.toml` and check them with `just util-structure-preview <name>`; `docs/STRUCTURES.md` has the format and an AI prompt workflow, and the `author-structure` skill walks an agent through it.

- This game renders using ASCII text. For how sprites and art should look (Caves of Qud, Metroid and CICADAMATA), follow `docs/ART-DIRECTION.md` when you add or change any sprite, art or item description.

- `docs/STEAM-MVP.md` is the Steam release plan, and `docs/SPEC-GAPS.md` lists my long-term ideas that have no OpenSpec change yet. By no means do we need to implement all of them. I just want to focus on the Steam MVP.

Periodic tasks:

- Prune our various docs/ documents and make sure tasks are marked as completed, specifically `docs/STEAM-MVP.md`.

- Make sure rust warnings get handled.

- Examine code for any TODOs and make sure they are still relevant.