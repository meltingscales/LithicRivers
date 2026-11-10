# Tasks

## 1. The bot

- [x] 1.1 Add `MiningBot` (registered for saving), `EntityKind::MiningBot`, `EnemyKind::MiningBot`, `world_spawn_mining_bot`, `MINING_BOT_LOOT` and `TestWorld::mining_bot`; make `spawner::embryo` return an `Option`; verify `every_component_is_registered`, the loot and embryo tests, and `cheat spawn_enemy mining_bot` in a headless test
- [x] 1.2 Add `mining_bot_system` (hearing, stepping, drilling into its inventory) and schedule it before the combat trigger; verify core tests for behind a wall, out of earshot, through rock into combat, keeping an iron scrap, and stone bricks holding
- [x] 1.3 Give bots their combat stats and a wreck on death with no embryo; verify core tests for health and the wreck's contents

## 2. Spawning and art

- [x] 2.1 Add `game::BUNKER_POS`, `SpawnRule::away_from_bunker` and the deep-band bot rule; verify spawn tests for deep only, not by the bunker, and far from the bunker
- [x] 2.2 Add the mining bot sprite and combat panel name; verify a snapshot script of a bot drilling toward the player and `bundled_fonts_cover_every_drawn_character`

## 3. Docs

- [x] 3.1 Update AGENTS.md (`spawn_enemy` kinds), `docs/STEAM-MVP.md` and `docs/SPEC-GAPS.md`; verify `just code-clippy` and `cargo test --workspace`
- [ ] 3.2 User check that bots feel dangerous but fair, that hearing them drill toward you is readable, and that their wrecks are worth fighting for
