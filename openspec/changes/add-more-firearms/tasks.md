# Tasks

## 1. Sulfur

- [x] 1.1 Sulfur slime: `EnemyKind::SulfurSlime`, spawn function with 1 to 3 seeded sulfur, the nuclear slime's movement shared through one slime system, combat stats, embryos in three grades, `spawn_enemy sulfur_slime`, and save registration; verify `cargo test` for its chase, health between a dog's and a nuclear slime's, loot and determinism, and a spawner growing one from its embryo
- [x] 1.2 `SpawnPlace::NearLava` and the sulfur slime's natural spawn rule; verify `cargo test` that they spawn near lava in the deep and never in the shallows, within the cap
- [x] 1.3 Sulfur vents: worldgen marks spots near lava in the deep, the game places vent tiles and entities, mining a vent refuses, and vents grow sulfur crystals to the cap while the player is near; crystals mine into sulfur; verify `cargo test` for placement (deep only, near lava, deterministic), unbreakable, growth, the cap, mining, and the surface hash unchanged

## 2. Gun traits

- [x] 2.1 `Gun` gains pellets, burst, stun chance, pierce, splash and `Ammo::Item` / `Ammo::Charge`; the pipe gun and rifle keep their behavior; verify the existing firearms tests still pass unchanged
- [x] 2.2 Shoot resolves each trait: per-pellet rolls, bursts that stop when rounds run out, seeded stun, charge drawn from the fullest cell (not offered when too low), piercing along the line to the first wall, splash on hit to enemies beside the target and never the robot; verify `cargo test` for each, and that shooting replays the same for the same seed

## 3. Guns and ammo

- [x] 3.1 Items, recipes and stats for the nail driver, scattergun, burst carbine, slug thrower, arc caster, riverglass lance and isotope launcher, and scatter shells, brass slugs and isotope canisters; verify the crafting reachability test and a test per gun's hit chance at 1 and 3 tiles

## 4. Client

- [x] 4.1 Sprites for every new gun, ammo, the sulfur slime, the vent and the crystal; Look text for vents; the combat panel's odds line for pellets, bursts and charge; verify the sprite gallery and font tests and a `just client-blind` snapshot of a scattergun and an arc caster fight
- [x] 4.2 A snapshot of a sulfur vent with grown crystals (`cheat set_tile` for lava beside a vent spot, then `wait`); verify the new snapshot

## 5. Checks

- [x] 5.0 A `scripts/bench/` scenario in the deep beside sulfur vents and lava, with a sulfur slime fight and several growth periods, so `just util-bench` keeps vent growth, near-lava spawn checks and the new Shoot paths inside the 16 ms tick and frame budget; verify `just util-bench` passes with the new row
  - `sulfur-vents` stays under 1 ms per tick and frame. `deep-underground` sometimes has one tick at 12 to 17 ms on `release` too, without this change (checked on a clean worktree), so it isn't from vents or the near-lava spawn check.

- [ ] 5.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 5.2 User check: each tier's guns feel different and worth switching to, and sulfur from vents and slimes is enough to keep gunpowder flowing

## Workflow follow-up

- Archive `add-firearms` before this change, since its `firearms` capability is what this change adds to.
- When merged with only 5.2 left, add a playtest queue entry for it.
