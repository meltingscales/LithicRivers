# Proposal

## Why

The game window (`add-game-window`) already draws with winit and wgpu, and both run on Android. A phone or tablet build would reach players without a PC and test the game on smaller screens, without changing the game itself. The user wants an Android target, shipped first as an APK to sideload; the Play Store comes later.

## What Changes

- **An Android build:** a signed APK, built locally with one `just` recipe and in CI, that runs the same game in the same window as on desktop.
- **The client as a library:** today all of the client's modules hang off `main.rs`. Android loads the game as a shared library, so the client becomes a library crate (`lib.rs`) with a thin desktop `main.rs`. Desktop behavior, headless runs, MCP and snapshots don't change.
- **Files in the app's storage:** config, saves, snapshots and logs go in Android's internal data folder instead of next to the executable or in the working folder.
- **Phone lifecycle:**
  - The game saves when the app goes to the background, since Android may close it there without warning.
  - It recreates its window surface when it comes back.
  - The Back button acts as Escape.
- **Touch controls (UI tweaks):**
  - An on-screen pad for movement and wait, plus buttons for Interact, Mine, Build mode, the menu tabs and Escape. The buttons send the same keys the keyboard does, so every keybind and screen works unchanged.
  - Tapping a cell is a mouse click, and dragging scrolls.
  - The soft keyboard opens for text fields (world name and seed, search, the cheat console).
  - The pad shows on touch devices by default, and Options can hide it, for example with a Bluetooth keyboard.
- **Landscape only, for now.** Portrait would need a different layout, so it's a later step.

## Capabilities

### New Capabilities
- `android-build`: building and signing the APK, where the game keeps its files on Android, and the app lifecycle (save on background, surface recreated on resume, Back as Escape).
- `touch-controls`: the on-screen pad and buttons, tap and drag input, the soft keyboard for text fields, and the Options setting that shows or hides the pad.

### Modified Capabilities
None. Desktop behavior is unchanged; the game window gains touch input only where there is a touch screen.

## Impact

- **Crates:**
  - `crates/client` gains `lib.rs`, and the modules move behind it. `main.rs` keeps the argument handling and the desktop and terminal entry points.
  - A `cdylib` target for Android, with an `android_main` entry point.
- **Dependencies:**
  - winit's `android-game-activity` feature (GameActivity is needed for soft-keyboard text input; NativeActivity has none).
  - `cpal`'s Android backend (it needs the NDK).
  - No new crates for touch.
- **Build:**
  - an `android/` Gradle wrapper project that hosts GameActivity;
  - `cargo-ndk` to build the Rust library;
  - `just build-android` and an Android job in `release.yml` that uploads the APK;
  - a debug signing key for sideloading, with the release keystore supplied as a CI secret later.
- **Core:** no changes beyond where config and saves are rooted (`ConfigManager` takes an optional base folder).
- **Docs:** AGENTS.md (building for Android) and `docs/STEAM-MVP.md` (Android as a post-MVP platform).
