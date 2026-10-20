# Design

## Context

- Startup today: `App::build` creates `Game::new(seed)` immediately (seed from `DEFAULT_SEED`, or the script/MCP seed), then `SplashState` runs Logo → GameTitle → BootMessage → MainUI. Splash input (`app/input/splash.rs`) skips each stage on any key.
- Save/load: the Menu tab's `ui.SAVE_JSON` / `ui.LOAD_JSON` keys write and read `save.json` in the working directory via `Game::save_json_with_viewport` / `load_json_with_viewport` (`SaveData`, JSON). `SAVES_FOLDER` (`lithicrivers-saves/saves`) exists in config but is unused.
- `SaveData` already covers seed, tick, tiles, player, sheep, feral dogs, items, NPCs, viewport, chunk/structure state, quests. Its gaps (corpses, body damage, and more) are addressed by `flawless-world-saves`.
- Developer mode is the global `game.DEVELOPER_MODE` config value; the tutorial is `app.ui.tutorial_visible`; the cheat console opens on `ui.OPEN_COMMAND_MENU` with no gating.
- Volumes are hard-coded in `App::build` (`0.60` music, `0.35` sfx). `ConfigManager::save()` can write the config file.
- Headless runs and MCP sessions build `App::new_headless(seed)` and `skip-splash` sets `SplashState::MainUI` on the already-created game. All 15 snapshots and 3 bug scripts depend on this.
- `Clock` provides `now()` and `animation_millis()`; the fake clock is deterministic.

## Goals / Non-Goals

**Goals:**
- Minecraft-style flow: title → world list → create/play, plus options.
- Several worlds, each a folder, each with its own settings.
- No change to any existing script or snapshot output.

**Non-Goals:**
- Fixing `SaveData` gaps (corpses, body damage, tutorial progress, etc.); that is the separate `flawless-world-saves` change. Switching to msgpack is also out of scope.
- Importing the old `save.json`.
- Autosave, world rename/duplicate, world thumbnails, game-mode/difficulty settings, keybind editing in Options.
- Steam Cloud or OS-specific save locations (`SAVES_FOLDER` stays relative to the working directory, as `save.json` was).

## Decisions

### Screens as `SplashState` variants
Add `Title`, `WorldList`, `CreateWorld`, `ConfirmDelete` and `Options` to `SplashState` (renamed in place to a front-end state, keeping the existing variants). `ui()` already branches on `SplashState` before drawing the main UI, and splash input is already handled first in `handle_input`, so the menu slots in without touching the in-game UI. Flow: Logo → GameTitle → **Title**; creating a world goes CreateWorld → BootMessage → MainUI; playing a saved world goes straight to MainUI.
Alternative: a separate top-level "screen" enum beside `SplashState`. Rejected: two state machines for one "which full-screen thing is showing" question.

### `App` owns world lifecycle
- `App::start_world(settings)`: build `Game::new(settings.seed)`, apply settings to `Resources`, reset in-game UI state (tab, view, panels, combat, tutorial) to what `App::build` sets today, and record the current world folder.
- `App::load_world(folder)`: build a game for the saved seed, `load_json_with_viewport`, apply settings from metadata, reset UI state, restore viewport.
- `App::save_world()`: write save data and metadata (last-played time) to the current folder.
While the title menu is showing, `app.core.game` holds `Game::new_empty(seed, origin)` (cheap, no worldgen, never ticked) so the rest of the client can keep assuming a game exists.

### World folders
`<SAVES_FOLDER>/<folder>/world.json` (existing `SaveData` format) and `meta.json`:
`{ "version": 1, "name", "seed", "cheats_allowed", "developer_mode", "show_tutorial", "created_unix", "last_played_unix" }`.
Folder name: the world name lowercased, non-alphanumerics replaced with `-`, plus `-2`, `-3`, ... if taken. Listing reads each folder's `meta.json`; a folder whose files fail to parse is listed as unreadable (deletable, not playable).

### Seeds
Blank: derived from the current wall-clock time via the app clock (real clock: system time; fake clock: fixed, so headless is deterministic). Numeric text: parsed as `u64`. Other text: FNV-1a 64-bit hash of the UTF-8 bytes, a fixed algorithm so the same text always gives the same seed across builds and platforms (unlike `DefaultHasher`).

### Wall-clock times through `Clock`
Add `Clock::unix_seconds()`: system time for the real clock, a fixed epoch plus the fake offset for the fake clock. Used for blank seeds, created/last-played times and the splash-text pick, keeping headless output byte-identical.

### Per-world settings in `Resources`
Add `WorldSettings { cheats_allowed, developer_mode, show_tutorial }` to core `Resources` (defaulting from config). `GameConfig::developer_mode` stays as the default for new worlds; in-game checks read the world's setting. The cheat-console open handler checks `cheats_allowed`; the headless `cheat` step and MCP call the registry directly and are not gated.

### Headless and MCP
- `App::new_headless(seed)` starts on the title screen with the placeholder game. `skip-splash` calls `start_world` with the run's seed, cheats allowed and config defaults, then shows MainUI. That reproduces today's state, so existing scripts and snapshots don't change; this is verified, not assumed (task 1.4).
- Headless runs set `SAVES_FOLDER` to a fresh directory under the system temp dir, removed when the `Runner` is dropped.

### Keybinds
New `title` category in `default_config.rs`: `UP` (`UP`), `DOWN` (`DOWN`), `SELECT` (`ENTER`), `BACK` (`ESCAPE`), `LEFT`/`RIGHT` (`LEFT`/`RIGHT`, for toggles and volume). Text fields (name, seed) take typed characters and `BACKSPACE`. In-game: `ui.SAVE_AND_QUIT_TO_TITLE` (`Q` in the Menu tab).

### Splash texts
`assets/config/splashes.txt`, one per line. Pick index = `unix_seconds % count` when the title screen opens (fixed in headless runs).

## Risks / Trade-offs

- [Existing snapshots change because the game is now created later] → `start_world` must leave the game and UI exactly as `App::build` did; task 1.4 runs the full suite before any menu work, and no `.snap` file may change in this PR except new title-menu ones.
- [Two game constructions at startup in headless runs (placeholder + real)] → The placeholder is `new_empty`, which does no world generation; negligible.
- [Deleting a world folder is destructive] → Confirmation screen; delete only paths inside the saves folder, refuse otherwise.
- [`SaveData` gaps mean a reloaded world isn't perfect (e.g. corpses vanish)] → Fixed by `flawless-world-saves`; this change stores `world.json` with whatever save format is current, so the two can land in either order.
- [Overlap with `split-client-main`] → Land that refactor first if possible; otherwise this change's new code still goes in new `ui/title/` files, and the conflict is limited to `ui()` and `SplashState`.
