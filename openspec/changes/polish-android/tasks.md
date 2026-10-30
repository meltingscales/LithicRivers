# Tasks

Install the APK from the latest GitHub Release, or build it with `just build-android release` and `adb install -r android/app/build/outputs/apk/release/app-release.apk`. Write down the phone model, Android version and keyboard app for each run. A failed check gets a note here and becomes a fix (or a change of its own if it's large).

## 1. Playtest on a phone

- [ ] 1.1 User check: the APK installs (from the release download, not only adb), opens on the title screen in landscape at a readable text size, and shows the keyboard tip
- [ ] 1.2 User check: with touch alone, start a new world, naming it with the soft keyboard; the keyboard opens on the name and seed fields and closes after
- [ ] 1.3 User check: with touch alone, move (all eight directions and wait), mine, build something from the hotbar, open the inventory and crafting tabs with the tab buttons, and accept and leave menus with Enter and Esc
- [ ] 1.4 User check: tapping a menu row selects it, and dragging scrolls a long list (inventory, crafting, the item dictionary)
- [ ] 1.5 User check: switching to another app and back, and turning the screen off and on, keep the game where it was; after the system closes the app in the background, the next launch loads the saved world
- [ ] 1.6 User check: Back does what Escape does (closes windows and menus) and never exits the app; Quit on the title screen closes it, and the next launch starts normally
- [ ] 1.7 User check: Options › Touch controls hides and shows the pad, and the choice survives a restart; Text size changes the size
- [ ] 1.8 User check: with a Bluetooth or USB keyboard (with a numpad if possible), movement, menus and typing work as on desktop
- [ ] 1.9 User check (optional): a second keyboard app (for example SwiftKey or the Samsung keyboard) types a world name correctly

## 2. Known polish

- [ ] 2.1 Decide with the user how the pad and the game share a phone screen (the pad covers the left of Messages and the first menu tabs; the map gets about 19 rows at 36px). Small layout tweaks land here; a phone-first layout becomes its own change
- [ ] 2.2 A launcher icon for the APK (adaptive icon resources under `android/app/src/main/res/`), in the game's art direction (`docs/ART-DIRECTION.md`)
- [ ] 2.3 Shrink the APK: strip symbols from the Android libraries (and consider LTO for them); verify the APK size before and after, and that the game still starts
- [ ] 2.4 Release signing: create a release keystore, add the four `LITHICRIVERS_*` secrets that `android-apk.yml` reads, and check whether Play Protect still flags the release APK
- [ ] 2.5 Fix what the playtest in section 1 turns up, one task per finding

## Workflow follow-up

- [ ] When section 1 passes, archive this change and update the Android line in `docs/STEAM-MVP.md`
