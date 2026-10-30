# Design

## Context

- The desktop client starts in `run_desktop` (`crates/client/src/lib.rs`), which parses headless flags, builds `App`, then opens the game window (winit + `ratatui-wgpu`, `crates/client/src/window/`) or, with `--terminal`, the terminal UI. Both loops call the shared per-frame `step(app)`. Headless runs and MCP build `App` through their own paths and never reach the window.
- Saves live in `data_path(SAVES_FOLDER)`, default `lithicrivers-saves/saves`. On desktop no base folder is set, so this is relative to the working directory; Steam launches games with the install folder as the working directory.
- Death: `lithicrivers_core::death::player_died` marks the player `Dead`; `death::is_dead(world, player)` reads it, and the system-failure modal stays up until respawn. `res.world_settings.cheats_allowed` says whether the world allows cheats (off by default for worlds made on the title screen, on for `skip-splash` and debug worlds).
- Release CI builds Linux, Windows and macOS, stages each into `artifacts/`, and `app_build_unified.vdf` uploads all three into one depot (3940721) of app 3940720.

## Goals / Non-Goals

**Goals:**
- Development builds, tests, headless runs and Android are untouched: no Steam library at link or run time.
- The achievement trigger is testable with `cargo test`, without Steam.

**Non-Goals:**
- Reacting to the overlay opening (pausing the game on `GameOverlayActivated`).
- Retrying unlocks earned while Steam was unavailable.
- Using the ISteamRemoteStorage API.

## Decisions

### A `steam` cargo feature, on only for release builds
The `steamworks` crate links `libsteam_api` dynamically, and a binary linked against it fails to start where the library is missing. So the dependency is optional behind a `steam` feature on `lithicrivers-client`, enabled by the `build-release-*` recipes only. `cargo test --workspace`, `just build-linux`, `client-blind`/MCP and `build-android` stay as they are. CI's release tests run with the feature on, so it is compiled and linted there; `just code-clippy` also gets a `--features steam` pass on desktop.
- Alternative: always link and load the library lazily (`dlopen`). More code and unsafe for no gain while the release build is the only Steam build.

### One module, a stub without the feature
`crates/client/src/steam.rs` owns everything Steam. With the feature it wraps `Option<steamworks::Client>`; without it, the same type is an empty stub with the same methods, so callers have no `cfg`. `App` holds it as `app.steam`, default disconnected, so headless and MCP runs never connect.
- `steam::connect()` calls `Client::init_app(3940720)`; on error it logs "Steam unavailable: …" and returns the disconnected value. Using `init_app` with the app id means no `steam_appid.txt` has to be shipped or kept next to dev binaries.
- `run_desktop` calls it after headless flags are handled and **before** the window and its wgpu device are created, because the overlay hooks the graphics API at device creation. The terminal UI connects too (achievements and cloud work; the overlay can't draw over a terminal).
- `step` calls `app.steam.run_callbacks()` each frame.

### Achievements: the client notices, Steam only records
Unlock conditions are checked in the client, where both the game state and the Steam handle are, rather than in core, so core stays free of platform code and deterministic.
- An `Achievement` enum in `steam.rs` with one variant, `FirstDeath`, mapped to the Steamworks API name `ACH_FIRST_DEATH`. `add-steam-achievements` grows this enum.
- `App` keeps `was_dead: bool`, reset whenever a world is loaded or created to that world's current state (so loading a world saved on the system-failure screen doesn't count as a new death). `step` compares it with `death::is_dead` and on the false→true edge calls `app.steam.unlock(Achievement::FirstDeath)` unless `cheats_allowed`.
- `unlock` records the achievement in `app.steam.unlocked_this_session` (always, even when disconnected or built without the feature, so tests can see it), and when connected calls `user_stats().achievement(name).set()` then `store_stats()`. Steam itself ignores repeated unlocks, which covers "at most once per account".
- Alternative: a core `GameEvent` for death that the client consumes. Nicer once many achievements exist; `add-steam-achievements` can introduce it if the edge checks multiply.

### Cloud saves through Auto-Cloud
Steam Auto-Cloud syncs a folder without code: in Steamworks, root "App Install Directory", subdirectory `lithicrivers-saves/saves`, pattern `*`, recursive, all OSes, with a quota sized from a large test world. The game already writes saves only on save and reads them on the world list, which is when Auto-Cloud syncs (at launch and exit).
- Alternative: ISteamRemoteStorage file API. Needs save code to know about Steam and handle conflicts itself; Auto-Cloud does it for free.

### Shipping the library
Each platform's artifact gets its Steam API library next to the executable: `libsteam_api.so` (Linux), `steam_api64.dll` (Windows), `libsteam_api.dylib` (macOS). `steamworks-sys` bundles these in its SDK copy; the stage scripts copy them from wherever the build leaves them (task 1.3 finds the exact path). Linux and macOS binaries get an rpath to their own folder (`$ORIGIN` / `@executable_path`) from a client `build.rs` that only emits it when `CARGO_FEATURE_STEAM` is set, so the library is found without `LD_LIBRARY_PATH` or the launcher script. The three file names differ, so they coexist in the shared depot.

## Risks / Trade-offs

- [The overlay doesn't hook wgpu's backend (Vulkan on Linux, DX12 on Windows, Metal on macOS)] → Launch from Steam on each platform (user check). If one fails, try creating the wgpu instance with a different backend on that platform (e.g. Vulkan or GL) before giving up on the overlay there.
- [Shift+Tab conflicts with a game binding] → No current binding uses Shift+Tab; players can change the overlay key in Steam anyway.
- [Saves inside the install folder are lost on uninstall or "verify files"] → Auto-Cloud restores them; moving saves to a per-user folder is a separate change (it would change `data_path` for every desktop player).
- [The game launched outside Steam from another working directory writes saves elsewhere] → Unchanged from today; Steam launches always use the install folder.
- [Release-only feature means Steam code isn't built in everyday `cargo test`] → The `--features steam` clippy pass and the release CI job compile it; the achievement logic itself is in the stub-shared path and tested without the feature.

## Migration Plan

Ship in a normal release. Existing local saves are already in the synced folder, so the first launch from Steam uploads them. Rollback: build without the feature; Steamworks settings can stay.

## Open Questions

- Achievement icon art (locked and unlocked, 256x256) for "System Failure": a placeholder is fine until `add-steam-achievements`.
