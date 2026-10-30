# Proposal

## Why

`add-android-build` was merged as a rough draft. The APK installs and runs on a Pixel 8, and the touch pad, soft keyboard and lifecycle each passed an agent's check over adb, but nobody has played a real session on a phone yet. The user will playtest it by hand. This change holds that playtest (taken over from `add-android-build`'s user check 4.2) and the polish found so far, so the rough edges are tracked in one place.

## What Changes

- **A manual playtest checklist** for the Android build: installing, playing with touch alone, typing, the lifecycle, and a hardware keyboard.
- **Polish already known from the agent's adb checks:**
  - The touch pad covers the left end of the Messages box and the first menu tabs. The game area is cramped at the phone text size (about 19 map rows), and the side panels take a lot of it.
  - The APK uses Android's default launcher icon.
  - The APK is about 138 MB, with each native library at about 68 MB unstripped.
  - Google Play Protect flagged the debug-signed APK as harmful on install (it still ran). A release keystore in CI secrets should help.
  - Gboard works, but only through a workaround: the game reads GameActivity's text field each frame. Other keyboards are untested.
- A failed check or a polish item becomes a fix here when it's small. A larger one becomes its own change: for example a phone-first layout, which the user floated as context-specific UI for Android, or a keyboard drawn by the game in place of the system one.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None for the checklist, which verifies behavior already in `android-build` and `touch-controls`, so this change sets `skip_specs`. A polish item that changes specified behavior (for example the pad's layout) goes in its own change with a spec delta.

## Impact

- Small fixes in `crates/client/src/window/` (touch pad, soft keyboard), the `android/` project (icon, signing) and the build recipe (APK size).
- When the playtest passes, archive this change.
