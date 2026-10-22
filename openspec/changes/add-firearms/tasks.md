# Tasks

## 0. Decisions

- [ ] 0.1 User decision: equipment or hotbar slot for guns, or the best gun in the inventory; record it in design.md

## 1. Materials

- [ ] 1.1 Nitre, sulfur and brass scrap deposits with drops; verify the worldgen survey pins and the "nothing unmineable" test
- [ ] 1.2 The items and recipes (gunpowder, paper, iron shot, casings, cartridges, both guns) and brass smelting; verify the reachability test and a furnace test for brass

## 2. Shooting

- [ ] 2.1 `MoveType::Shoot` with each gun's numbers, ammo use and casings returned; verify `cargo test` for hit chances, ammo, casings and deterministic replays
- [ ] 2.2 Military android ammo as brass cartridges; verify its loot test

## 3. Client and checks

- [ ] 3.1 The Shoot move with its hit chance in the combat panel, and sprites for every new item and deposit; verify a combat snapshot and the sprite gallery
- [ ] 3.2 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 3.3 User check that making ammo feels worth it and shooting feels good
