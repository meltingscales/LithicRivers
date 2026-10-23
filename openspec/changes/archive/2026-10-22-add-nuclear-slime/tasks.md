# Tasks

## 1. Loot and items

- [x] 1.1 Replace loot tuples with `CreatureLoot` (chance, min, max) and add `ItemKind::NuclearWaste` if missing; verify dog inventories are unchanged (golden entity test, save-reload identity test) and `raw_sources()` includes nuclear waste
- [x] 1.2 Leave a corpse for any dying creature that carries items; verify existing dog corpse tests pass and a creature with an empty inventory leaves no corpse

## 2. The slime

- [x] 2.1 Add `EnemyKind::NuclearSlime`, `EntityKind::NuclearSlime`, `NuclearSlime` and `SlimeAI` (registered for saving), spawn code and stats; verify tests for health above a dog's, the 5% drop rate band over 2000 spawns, and same-seed drops
- [x] 2.2 Add the slime AI (notice within 6 tiles, move every other turn) and combat stats; verify `TestWorld` tests for the slow chase (adjacent after 6 turns from 4 tiles) and a deterministic fight replay
- [x] 2.3 Add seeded underground spawning; verify no surface slimes and at least one underground slime over a fixed area
- [x] 2.4 Add `spawn_enemy nuclear_slime`; verify a headless test for the cheat and the updated "unknown kind" message

## 3. Client and checks

- [x] 3.1 Slime and nuclear waste sprites; verify a snapshot script of combat with a slime (spawned by cheat)
- [x] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [x] 3.3 User check that the slime feels different from a dog
