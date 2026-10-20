# Proposal

## Why

Starting the game drops straight into one hard-coded world (the configured `DEFAULT_SEED`) after the splash screens. Saving writes a single `save.json` into the working directory from the in-game Menu tab, so there is one slot, no way to pick a seed, no way to start over without deleting files, and the `SAVES_FOLDER` setting is unused. A Steam release needs the standard front door players expect: a title screen, a list of worlds, and a "create world" screen, in the style of Minecraft.

## What Changes

- **Title screen** after the logo/game-title splashes: the LithicRivers logo, a yellow "splash text" line picked from a list, and **Play Game**, **Options**, **Quit**.
- **World list** (Play Game): saved worlds sorted by last played, showing name, seed and last-played date, with **Play Selected World**, **Create New World**, **Delete** (with confirmation) and **Back**.
- **Create New World** screen: world name, seed (blank = new seed; numbers used as-is; any other text hashed to a seed, like Minecraft), **Allow cheats** (on/off), **Developer mode** (on/off), **Show tutorial** (on/off), then **Create** / **Cancel**.
- **Worlds on disk**: each world is a folder under `SAVES_FOLDER` holding its save data and a small metadata file (name, seed, settings, created/last-played times). Creating a world plays the existing boot-message intro, then starts the game.
- **In game**: the Menu tab's Save writes to the current world's folder; a new **Save and Quit to Title** action returns to the title screen. The old single `save.json` slot is retired (an existing `save.json` is left on disk, not imported).
- **Per-world settings take effect**: with cheats off, the cheat console doesn't open; developer mode and tutorial visibility come from the world instead of the global config.
- **Options** screen: music and sound volume, saved to the config file (they are hard-coded today).
- **Headless runs and MCP keep working unchanged**: `skip-splash` still starts the default world directly (with cheats on), and headless runs use a fresh, empty saves folder so the menus render the same every time. Every existing script and snapshot keeps passing.

## Capabilities

### New Capabilities
- `title-menu`: The title screen, its splash text, navigation between title, world list, create-world and options screens, and the options screen.
- `world-saves`: Worlds as named, separately saved folders: listing, creating with settings, loading, saving, deleting, and how per-world settings apply in game.

### Modified Capabilities
- `headless-client`: `skip-splash` starts the default world (seed from the script) instead of relying on a world created at startup; headless runs get an isolated, empty saves folder.
- `cheat-console`: cheat commands are available only in worlds that allow cheats.

## Impact

- Client: new `ui/title/` screens and input handling, `SplashState` gains title-menu states, `App` gains `start_world` / `load_world` / `save_world`, and the Menu-tab save/load handlers change. Builds on the `split-client-main` refactor if that lands first (new code goes under `ui/` either way).
- Core: per-world settings (`cheats_allowed`, `developer_mode`, `show_tutorial`) on `Resources`. Settings live in the world's metadata file, so the existing save-data format is unchanged.
- Config: new `title` keybind category, `audio` volume settings, `SAVES_FOLDER` now used.
- Assets: `config/splashes.txt`.
- Tests: new headless snapshots for the title, world list, create-world and options screens; world save/load round-trip tests.
