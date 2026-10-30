# Tasks

## 1. Dependency and build

- [ ] 1.1 Add `steamworks` (current release) as an optional dependency of `lithicrivers-client` behind a `steam` feature; verify `cargo test --workspace` still builds without it and `cargo build -p lithicrivers-client --release --features steam` builds on Linux
- [ ] 1.2 Add a client `build.rs` that, only when `CARGO_FEATURE_STEAM` is set, gives the `lithicrivers-client` binary an rpath to its own folder (`$ORIGIN` on Linux, `@executable_path` on macOS); verify with `readelf -d target/release/lithicrivers-client | grep -i runpath` after a `--features steam` build
- [ ] 1.3 Find where the `steamworks-sys` build leaves the Steam API library for each platform, and copy it into `artifacts/` in `scripts/stage-artifacts-release.sh`, `scripts/stage-artifacts-release.ps1` and the macOS step of `.github/workflows/release.yml`; verify `just stage-artifacts-release-linux` puts `libsteam_api.so` beside `lithicrivers-client` and `ldd artifacts/lithicrivers-client` resolves it
- [ ] 1.4 Enable `--features steam` in the `build-release-*` recipes (and the macOS release build in CI) and add a `--features steam` pass to `just code-clippy` on desktop; verify `just code-clippy` passes

## 2. Connecting to Steam

- [ ] 2.1 Add `crates/client/src/steam.rs`: a `Steam` type wrapping `Option<steamworks::Client>` with the feature and an empty stub without it, with `connect()`, `run_callbacks()`, `is_connected()`; `connect()` uses `Client::init_app(3940720)` and logs "Steam unavailable" on error. Add `app.steam` (disconnected by default); verify `cargo test --workspace` and `just code-clippy`
- [ ] 2.2 In `run_desktop`, connect after headless flags are handled and before the window (and its wgpu device) or terminal UI starts; call `app.steam.run_callbacks()` from `step`; verify `just client-blind scripts/agent/smoke.txt` output is unchanged and the existing snapshot tests pass
- [ ] 2.3 Verify a `--features steam` release build with Steam closed reaches the title screen and logs "Steam unavailable" (user check: launch it with Steam closed)

## 3. Placeholder achievement

- [ ] 3.1 Add the `Achievement` enum (`FirstDeath` → `ACH_FIRST_DEATH`) and `Steam::unlock`, which records into `unlocked_this_session` and, when connected, sets the achievement and stores stats; verify `cargo test -p lithicrivers-client`
- [ ] 3.2 Track `app.was_dead`, reset to the world's current state on every world create and load, and unlock `FirstDeath` in `step` on the false→true edge unless `cheats_allowed`; add client tests: dying with cheats off records `FirstDeath` once (dying twice still records it once per edge, with no error), dying with cheats on records nothing, and loading a world saved while dead records nothing. Verify `cargo test -p lithicrivers-client`
- [ ] 3.3 Confirm `save_reload_anywhere_changes_nothing` and all snapshot tests still pass (`was_dead` and `app.steam` are not saved; `cargo test --workspace`)

## 4. Steamworks setup (manual, user)

- [ ] 4.1 In Steamworks, add achievement `ACH_FIRST_DEATH`, display name "System Failure", description "Lose your body for the first time.", with placeholder icons, and publish; user check: it shows in the app's Stats & Achievements page
- [ ] 4.2 Configure Auto-Cloud: root App Install Directory, subdirectory `lithicrivers-saves/saves`, pattern `*`, recursive, all OSes, a quota above a large test world's size; enable Steam Cloud for the app and publish; user check: the Steam Cloud settings page shows the rule

## 5. Integration checks (user, on a Steam release build)

- [ ] 5.1 Launch from the Steam library on Linux, Windows and macOS and press Shift+Tab in the game window; user check: the overlay opens on each (if one fails, see design.md Risks)
- [ ] 5.2 Die in a world without cheats; user check: "System Failure" unlocks; die in a world with cheats allowed on a fresh test account or after resetting the achievement; user check: nothing unlocks
- [ ] 5.3 Save a world on one computer, quit, launch from Steam on another; user check: the world is listed and loads

## 6. Docs

- [ ] 6.1 Check off the Steam Integration items covered in `docs/STEAM-MVP.md` (SDK integration, cloud saves, overlay; achievements stays open for `add-steam-achievements`) and add a short "Steam" section to `AGENTS.md` (the `steam` feature, release-only, how to test the achievement logic without Steam); verify by reading the diff

## Workflow follow-up

- Archive this change only after the user reports the section 5 checks passed, in its own PR.
