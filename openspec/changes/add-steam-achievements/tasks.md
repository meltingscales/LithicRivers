# Tasks

## 1. Stats

- [x] 1.1 Add `PlayerStats { enemies_killed, teleports }` to `Resources` (serde default); count non-player deaths in `handle_entity_death` and successful jumps in `Game::teleport`; verify core tests for counting kills and jumps, and a save/reload test of the counts

## 2. Achievements

- [x] 2.1 Add the five achievements to `steam::Achievement`, check them in `App::check_achievements`, and make `Steam::unlock` skip achievements already unlocked this session; verify client tests for Daylight (once over many frames), Self-Defense, Two Hands, Transit and nothing with cheats on
- [x] 2.2 Update AGENTS.md's Steam section and `docs/STEAM-MVP.md`; remove the achievements row from `docs/SPEC-GAPS.md`; verify `just code-clippy` and `cargo test --workspace`

## 3. Steamworks (manual, user)

- [ ] 3.1 In Steamworks, add `ACH_SURFACE` "Daylight" ("Reach the surface."), `ACH_FIRST_KILL` "Self-Defense" ("Destroy an enemy."), `ACH_REPAIR_ANDROID` "Spare Parts" ("Repair the broken android."), `ACH_REPLACE_ARM` "Two Hands" ("Get a new arm.") and `ACH_FIRST_JUMP` "Transit" ("Teleport for the first time."), with placeholder icons, and publish; user check: they show in the app's Stats & Achievements page
- [ ] 3.2 User check on a Steam release build in a world without cheats: escaping the bunker unlocks Daylight
