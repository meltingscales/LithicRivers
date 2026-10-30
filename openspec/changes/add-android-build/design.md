# Design

## Context

- The client is a binary crate: `main.rs` declares every module (`app`, `ui`, `window`, `headless`, ...) and holds `step`, `handle_input`, `handle_mouse_event` and the argument handling.
- The game window (`crates/client/src/window/`) runs a winit `ApplicationHandler` that drives a `ratatui-wgpu` backend (our fork). Input is already translated at that edge: window key and mouse events become crossterm `KeyCode`s and screen cells.
- `ConfigManager::resolve_config_path` roots the config at `<exe_dir>/config`. Logs go to `.`, and saves to the relative `lithicrivers-saves/...` from the config.
- Assets are embedded with `rust-embed`, so the binary needs no asset files at runtime.
- Audio is rodio on cpal. cpal 0.15 has an Android backend through oboe, which needs the NDK's C++ toolchain.

## Goals / Non-Goals

**Goals:**
- One APK built by a recipe and by CI, running the unchanged game.
- Playable with touch alone, with no game logic changed: the touch layer produces the same keys and clicks as a keyboard and mouse.

**Non-Goals:**
- The Play Store: an AAB bundle, the target-API upkeep, the listing and release keystore management. That's a later change.
- Portrait layout, gamepads (`add-controller-support` in SPEC-GAPS), iOS.
- Performance work beyond measuring: `just util-bench`'s numbers on a phone are recorded, and fixes go in their own change.

## Decisions

**Split the client into a library and a thin binary.** `lib.rs` declares the modules and exposes:
- `run_desktop(args)`: today's `main` body;
- the frontend functions (`step`, `handle_input`, `handle_mouse_event`, `ui`).

`main.rs` becomes a few lines calling `run_desktop`. The Android `cdylib` entry (`android_main`) lives in the same library behind `cfg(target_os = "android")`. We considered a separate `crates/android` crate, but it can't depend on a binary crate, and duplicating the module tree would be worse. The split moves code without changing it, so snapshots and the `save_reload_anywhere` test prove nothing changed.

**No `cdylib` in `Cargo.toml`.** The desktop build would also make a `.so`/`.dll`, and on Windows its `.pdb` would collide with the binary's. Instead, `cargo ndk ... rustc --lib --crate-type cdylib` asks for the shared library only when building for Android.

**GameActivity through winit, hosted by a small Gradle project.** winit's `android-game-activity` feature gives soft-keyboard text input (IME), which the text fields need. NativeActivity has no IME. GameActivity needs its Java AAR, so `android/` holds a minimal Gradle project:
- one activity;
- a landscape-only manifest;
- the debug signing config.

`cargo-ndk` builds `liblithicrivers_client.so` for `arm64-v8a` and `x86_64` into `jniLibs`. `just build-android` runs `cargo ndk` and then `./gradlew assembleDebug` (or `assembleRelease` with a keystore from environment variables). We considered `xbuild` and `cargo-apk`. They're simpler, but `cargo-apk` is unmaintained, and neither makes the Play Store's AAB, which Gradle does once that later change needs it.

**Storage root from the platform.** On Android, `android_main` reads `AndroidApp::internal_data_path()`, and the client passes it as the base folder for:
- `ConfigManager` (a new optional base path, defaulting to today's executable folder);
- the log appender;
- the saves and snapshots folders, which stay relative to the base.

Desktop passes nothing and behaves as today.

**Lifecycle in the window module.**
- **Suspended:** winit's `suspended` drops the GPU surface and saves the current world through the existing save path, reusing what Save and quit does.
- **Resumed:** `resumed` recreates the surface (the window module already creates it lazily there), and the game continues.
- **Back:** with target SDK 36, Back is a predictive-back gesture rather than a key event, so a `MainActivity` subclass of GameActivity registers a back callback that sends Escape to the game. Leaving the app is Home or Quit.

**Touch layer next to the existing input translation.** It lives in `window::touch`, in pure, tested functions:
- **On-screen pad:** laid out in cells along the bottom of the game area. Movement sits on the left (a 3x3 grid with wait in the middle) and buttons on the right (Interact, Mine, Build, ◀ tab, ▶ tab, Esc). Each button names a keybind (`movement.MOVE_EAST`, `action.INTERACT`, ...), and a press sends that keybind's first key through `handle_input`. Rebinding therefore just works, and the labels show the bound key.
- **Space for the pad:** like the docked Display panel, the game draws into an in-memory screen sized to the rest of the window, so its coordinates don't change.
- **Taps and drags:** a tap (a touch that ends within one cell) is a left click on that cell. A drag past one cell becomes scroll steps.
- **Soft keyboard:** shown with `set_ime_allowed(true)` while any text field takes typing, on mobile only. A small `App::typing()` check covers the title fields, search, the cheat console and blueprint naming. *Outcome (Pixel 8, Gboard):* the keyboard types into GameActivity's own text field rather than sending key presses (only Backspace and Enter arrive as keys), and winit 0.30 drops that text. So each frame the window reads the field through the `AndroidApp`, types its characters and empties it. The editor is set to visible-password text with no suggestions, so the keyboard never holds back a half-typed word, and to no-fullscreen, so landscape keeps the game visible. Select on a title text field moves to the next field, since the keyboard covers the rows below. If keyboards prove unreliable, the fallback is a small keyboard drawn by the game.

Touch is mobile-only: `android_main` sets a runtime `mobile` flag (never saved), and only then does the window draw the pad or read touch events. Desktop assumes a keyboard, even on a touch screen. Options gains "Touch controls: shown / hidden", saved under `ui.TOUCH_CONTROLS`, which reads "only on mobile" on desktop. The pad is laid out below the game, in the game's column when the Display panel is docked.

**Landscape only, with the default text size raised on small screens.** On first launch on Android, the text size is chosen so the game area is at least 80 columns wide. The docked Display panel is off by default on phones (hidden), because the pad uses the space and Options still has Text size.

## Risks / Trade-offs

- [cpal's oboe backend needs the NDK C++ toolchain and can fail to build] → The first task builds the library with audio on. If oboe fails, audio is turned off on Android behind a feature, with a follow-up change, rather than blocking the build. *Outcome (task 2.1):* with NDK 28, the library builds for `arm64-v8a` and `x86_64` with audio on. cpal links Android's OpenSL ES, so no feature flag was needed. The `.so` exports `android_main` and `GameActivity_onCreate` and needs only system libraries (`liblog`, `libandroid`, `libOpenSLES`, libc).
- [The ratatui-wgpu fork on Android's Vulkan drivers] → The emulator smoke run and a real device both draw the title screen before the rest of the work.
- [A crossterm dependency in core, `keycode_mapping`] → It's only types on Android, and crossterm builds for Android targets. The terminal frontend (`--terminal`) is compiled out there.
- [Losing progress when Android kills the app] → The world saves on every suspend, and the test drives the suspend path headlessly through the same save function.
- [GameActivity may still finish the activity on Back] → winit delivers Back as `BrowserBack`, which becomes Escape. If GameActivity's default back handling also closes the app, the fix is a small Java subclass that overrides `onBackPressed`. That's checked on a device in 4.2. *Outcome (task 2.2, Pixel 8):* GameActivity did finish on Back, and because winit allows one event loop per process, the next launch hung on the splash icon. `MainActivity` now turns Back into Escape, and `android_main` ends the process when the game loop returns. The device run also showed that winit on Android reports Enter, Tab and Escape as control characters (`"\n"`, `"\t"`, `"\u{1b}"`), which the input translation now maps to their keys.
- [I can't run a phone here] → Pure functions (touch layout, tap and drag, the storage root) are unit-tested, the build runs in CI, and playing on a device is a user check.

## Migration Plan

Desktop users see no change. The library split lands first, as its own commit with snapshots unchanged, before any Android code.
