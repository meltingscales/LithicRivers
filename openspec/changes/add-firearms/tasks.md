# Tasks

## 0. Decisions

- [x] 0.1 User decision, recorded in design.md: one weapon slot per working arm, and multi-arm builds fire every equipped gun in one turn

## 1. Materials

- [ ] 1.1 Nitre, sulfur and brass scrap deposits with drops; verify the worldgen survey pins and the "nothing unmineable" test
- [ ] 1.2 The items and recipes (gunpowder, paper, iron shot, casings, cartridges, both guns) and brass smelting; verify the reachability test and a furnace test for brass

## 2. Shooting

- [ ] 2.1 Weapon slots on arms (equip, unequip, losing a slot when an arm is destroyed, kept on wrecks); verify `cargo test` for each
- [ ] 2.2 `MoveType::Shoot` firing every equipped, loaded gun in one turn, with each gun's numbers, ammo use and casings returned; verify tests for hit chances, two and four guns firing at once, ammo, casings and deterministic replays
- [ ] 2.3 Military android ammo as brass cartridges; verify its loot test

## 3. Client and checks

- [ ] 3.1 Equipping in the Body tab, the Shoot move listing each gun's hit chance and ammo in the combat panel, and sprites for every new item and deposit; verify Body tab and combat snapshots and the sprite gallery
- [ ] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check that making ammo feels worth it and shooting feels good
