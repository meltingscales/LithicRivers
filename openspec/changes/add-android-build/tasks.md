# Tasks

## 1. Library split

- [x] 1.1 Move the client's modules behind `lib.rs` with `run_desktop`, keeping `main.rs` a thin entry; verify `just code-clippy`, `cargo test --workspace` with no snapshot changes, and `just client-blind scripts/agent/smoke.txt`
- [x] 1.2 An optional base folder for `ConfigManager`, the log appender, and the saves and snapshots folders (desktop passes none); verify unit tests that a base folder roots each path and that none keeps today's paths

## 2. Android build

- [x] 2.1 The `cdylib` target with `android_main`, winit's `android-game-activity`, and the internal data path as the base folder; verify `cargo ndk -t arm64-v8a build -p lithicrivers-client` succeeds (with audio, or audio behind a feature with the reason recorded in design.md)
- [x] 2.2 The `android/` Gradle project (GameActivity, landscape manifest, debug signing) and `just build-android`; verify the recipe produces an APK that installs and draws the title screen on the emulator (`adb install`, plus a screenshot)
- [ ] 2.3 An Android job in `release.yml` that builds the APK on each release tag and attaches it to the GitHub Release as `lithicrivers-<version>.apk` (a `github-release-android` job like the macOS one), with release signing from secrets when present; verify the workflow on a branch run
- [x] 2.4 Lifecycle: save on suspend, surface recreated on resume, Back as Escape; verify a headless test of the suspend save path and a unit test for the Back mapping

## 3. Touch controls

- [x] 3.1 `window::touch`: the pad layout (movement 3x3 with wait, Interact, Mine, Build, tab left and right, Esc), button hit-testing, labels from the bound keys, tap as a click and drag as scroll steps; verify unit tests for layout at several sizes, hit-testing, a rebound key, and tap versus drag
- [x] 3.2 Draw the pad beside the game (the game in an in-memory screen sized to the rest), only on mobile (a runtime flag `android_main` sets), shown by default there, with Options "Touch controls: shown / hidden" saved; verify an Options snapshot and a test of the setting round trip
- [x] 3.3 The soft keyboard while a text field takes typing (`App::typing()`), with IME commits as typed keys and deletes as Backspace; verify unit tests for `typing()` in each text field and for the IME mapping
- [x] 3.4 First-launch text size for small screens and the Display panel hidden by default on phones; verify a unit test choosing the text size for a phone-sized window
- [x] 3.5 On Android, the title screen's "Keyboard with numpad is recommended to play LithicRivers" line (a flag set by `android_main`, so headless runs and snapshots stay as they are); verify a headless test that turns the flag on and checks the line, plus the device screenshot

## 4. Checks

- [x] 4.1 Docs: AGENTS.md (building for Android, the library split) and `docs/STEAM-MVP.md` (Android after the MVP); verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 4.2 User check: on an Android phone, the APK installs, the game plays with touch alone (moving, mining, building, menus, naming a world), and progress survives switching away and back
