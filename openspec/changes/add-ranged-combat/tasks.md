# Tasks

## 1. Core

- [x] 1.1 `line_of_sight` shared with fog of war, and ranged creatures joining combat in range; verify `cargo test` for walls blocking and combat starting at range
- [x] 1.2 `EnemyShoot` with the hit chance falloff and ammo; verify tests for falloff, ammo use and deterministic replays
- [x] 1.3 The military android (AI, stats, loot, structure spawn tile, `spawn_enemy`); verify tests for keeping distance, closing in out of ammo, and its loot

## 2. Client

- [x] 2.1 Distance and ammo in the combat panel, the shot line, and the android's sprites; verify a combat snapshot and the sprite gallery

## 3. Checks

- [x] 3.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.2 User check that fighting an android is tense but fair
