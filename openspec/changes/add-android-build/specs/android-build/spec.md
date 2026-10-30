# Spec Delta

## Purpose

Building LithicRivers for Android as an installable APK, where the game keeps its files there, and how it behaves when the app is paused, resumed or closed.

## ADDED Requirements

### Requirement: Android APK
The project SHALL build a signed Android APK of the game with one recipe, locally and in CI, for 64-bit ARM phones and tablets (and x86_64 for the emulator). The APK SHALL run the same game, in the same window, as the desktop build.

#### Scenario: Building the APK
- **WHEN** a developer with the Android SDK and NDK runs the Android build recipe
- **THEN** it produces a signed APK that installs with `adb install`

#### Scenario: CI artifact
- **WHEN** the release workflow runs
- **THEN** it uploads the Android APK alongside the desktop builds

### Requirement: Files in the app's storage
On Android, the game SHALL keep its config, saves, snapshots and logs in the app's internal data folder, and SHALL NOT depend on the executable's folder or the working folder.

#### Scenario: Saves survive restarts
- **WHEN** the player saves a world, closes the app, and opens it again
- **THEN** the world is in the world list and loads

### Requirement: App lifecycle
When the app goes to the background, the game SHALL save the current world, and when it comes back, it SHALL redraw in a fresh window surface and continue where it was. The Android Back button SHALL act as the Escape key.

#### Scenario: Backgrounded and killed
- **WHEN** the player switches away from the game and Android closes it in the background
- **THEN** the next launch can load the world as it was when the player switched away

#### Scenario: Back button
- **WHEN** the player presses Back while a window or menu is open
- **THEN** it closes as Escape would

### Requirement: Desktop unchanged
Splitting the client into a library and adding the Android target SHALL NOT change the desktop client, the terminal mode, headless runs, MCP or snapshot output.

#### Scenario: Snapshots unchanged
- **WHEN** the test suite runs after the split
- **THEN** every snapshot matches and the desktop binary takes the same arguments as before
