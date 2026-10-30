# Proposal

## Why

LithicRivers ships on Steam (app 3940720), but the game doesn't talk to Steam at all: no overlay, no cloud saves, no achievements. `docs/STEAM-MVP.md` lists Steamworks SDK integration, achievements, cloud saves and the overlay as MVP items. The plumbing has to exist before any real achievements can be designed, so this change builds it end to end and proves it with one placeholder achievement.

## What Changes

- Link the Steamworks SDK into desktop release builds (Linux, Windows, macOS) behind a `steam` cargo feature, and ship the Steam API library next to the executable.
- Start Steam at launch, before the game window opens, so the Steam overlay can draw over the window. If Steam isn't running, or the build has no `steam` feature, the game runs exactly as it does today.
- Pump Steam callbacks once per frame.
- Cloud saves through Steam Auto-Cloud: the world saves folder is synced by Steam, configured in Steamworks. The save format and folder don't change.
- One placeholder achievement, "System Failure" (`ACH_FIRST_DEATH`), unlocked the first time the robot dies, to prove the achievement path works. Worlds with cheats allowed never unlock achievements.
- The real achievement list is out of scope: it gets its own change, `add-steam-achievements`, which replaces or keeps the placeholder.

Out of scope: Steam Workshop, rich presence, leaderboards, Steam Input / controller support (`add-controller-support`), DRM (`restart_app_if_necessary`), the SteamPipe upload itself (`new-steampipe-account-no-mfa`), Android.

## Capabilities

### New Capabilities
- `steam-integration`: how the client connects to Steam (startup, fallback when Steam is absent, overlay, cloud saves) and how achievements are unlocked.

### Modified Capabilities
<!-- None: dying and saving behave the same; Steam only observes them. -->

## Impact

- **Code:** new `crates/client/src/steam.rs`; startup in `crates/client/src/lib.rs` (`run_desktop`, before the window is created); the per-frame `step`; an `App` field for achievement state.
- **Dependencies:** the `steamworks` crate (with `steamworks-sys`, which bundles the SDK redistributables), only with the `steam` feature.
- **Build and release:** `justfile` release recipes enable `steam`; `scripts/stage-artifacts-release.sh` / `.ps1` and the macOS CI step copy `libsteam_api.so` / `steam_api64.dll` / `libsteam_api.dylib` into `artifacts/`; the Linux and macOS binaries get an rpath to their own folder.
- **Steamworks (manual):** create the achievement, configure Auto-Cloud, publish.
- **Docs:** `docs/STEAM-MVP.md` items checked off; the `docs/SPEC-GAPS.md` Steamworks row removed, with a new row for `add-steam-achievements`.
