# Proposal

## Why

`add-firearms` gives the robot two guns, both early to mid game, and nothing for the late game. Guns should change as the factory grows, and each should play differently, not just hit harder. Sulfur is also the scarcest part of gunpowder: it comes only from rare veins in the deep. More guns mean more gunpowder, so sulfur needs steadier sources first.

## What Changes

- **Five new guns, plus the existing two, across the game's tiers:**

  | Tier | Gun | What makes it different |
  |---|---|---|
  | Early | Pipe gun (exists) | Cheap, unreliable, short range |
  | Early | Nail driver | Spring-loaded, fires the nails you already make: weak, but no gunpowder |
  | Early | Scattergun | Several pellets per round, each rolling its own hit: strong up close, useless past 3 tiles |
  | Mid | Rifle (exists) | Accurate at long range |
  | Mid | Burst carbine | Fires 3 brass rounds per Shoot at lower accuracy: burns through ammo and casings |
  | Mid | Slug thrower | A heavy slug, slow and inaccurate, with a chance to stun what it hits |
  | Late | Arc caster | No cartridges: each shot drains charge from a carried power cell |
  | Late | Riverglass lance | A focused beam that pierces, hitting every creature on its line; powered by cells |
  | Late | Isotope launcher | Nuclear waste canisters that blast the target and every other enemy beside it |

- **New ammo:** scatter shells, brass slugs and isotope canisters, each with a recipe. The nail driver fires nails. The arc caster and lance use power-cell charge.
- **Gun traits** that the existing `Shoot` move gains: pellets per round, rounds per Shoot (bursts), a stun chance, piercing, splash, and charge ammo. These follow the existing rules for range, line of sight and seeded rolls.
- **Sulfur sources:**
  - **Sulfur vents:** fuming fissures that worldgen places in the deep near the lava rivers. They can't be broken, and they grow sulfur crystal tiles on the open ground around them over time. Mining the crystals gives sulfur, so it's a renewable supply worth building beside.
  - **Sulfur slimes:** a yellow slime of the deep's lava rivers. It always carries sulfur and drops sulfur slime embryos, so a spawner farm can produce sulfur.
  - The sulfur veins from `add-firearms` stay as they are.
- **Cheats:** `spawn_enemy sulfur_slime`.
- **Explosives:** the isotope launcher's blast lifts `add-firearms`' "no explosives" non-goal, for this one gun. Its splash hits enemies only, never the robot, and has no other effect on the world.

## Capabilities

### New Capabilities
- `sulfur-slime`: The sulfur slime: where it lives, how it fights, and what it carries and drops.
- `sulfur-vents`: Where sulfur vents appear, that they can't be broken, and how they grow mineable sulfur crystals.

### Modified Capabilities
- `firearms` (introduced by `add-firearms`; archive that change first): the five new guns, their ammo, and the gun traits (pellets, bursts, stun, piercing, splash, charge ammo).
- `cheat-console`: `spawn_enemy` supports `sulfur_slime`.
- `enemy-spawning`: sulfur slimes spawn naturally in the deep, and their embryos work in spawners.

## Impact

- **Core:**
  - The gun table and the Shoot resolution in `firearms.rs` gain the traits above.
  - Charge ammo draws from carried cells through the power-cell item data, the way the tele-access device pays for jumps.
  - New items and recipes, with the crafting reachability test covering them.
  - The sulfur slime: a spawn function, AI shared with the nuclear slime, loot, embryos and a spawn rule.
  - Sulfur vents: placed by worldgen and growing crystals on a schedule. Placement and growth are seeded from world seed, position and tick.
- **Client:** sprites for every new item, ammo, the slime and the vent; the combat panel's odds line handles pellets, bursts and charge; Look text for vents.
- **Saves:** new components are registered in `saved_components!`.
- **Depends on:** `add-firearms` and `add-ranged-combat` (both merged), `add-power-cells` (cell charge), and `add-molten-rivers` (lava rivers, for vent and slime placement).
