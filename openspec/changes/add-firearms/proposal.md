# Proposal

## Why

After `add-ranged-combat`, enemies can shoot, but the robot can only punch and tackle. The user's idea (IDEAS-INBOX.md) is guns with a real supply chain: gunpowder, paper, ammo, shell casings, and brass from nuggets to ingots. That turns fighting into a factory problem: ammo is something you make, and spent casings come back to be reloaded.

## What Changes

- **Two guns to start:**
  - a **pipe gun** (early, crafted from iron scrap and wood; fires paper cartridges; short range, poor accuracy)
  - a **rifle** (later, needs brass and the electric furnace tier; fires brass cartridges; long range, good accuracy)
- **Shooting in combat:** with a gun and matching ammo in the inventory, the combat move list gains "Shoot (gun)". It uses the same line of sight, accuracy falloff and seeded hit roll as enemy shots (`add-ranged-combat`), with each gun's own range, accuracy and damage.
- **The ammo chain:**

  | Item | From |
  |---|---|
  | Nitre | mined in the shallows (a new deposit) |
  | Sulfur | mined in the deep near the Lithic Rivers (a new deposit) |
  | Gunpowder | biochar + nitre + sulfur |
  | Paper | wooden shavings |
  | Iron shot | iron scrap |
  | Paper cartridge | paper + gunpowder + iron shot |
  | Brass nugget | salvaged from brass scrap deposits in the deep, and from military androids |
  | Brass ingot | brass nuggets smelted in a furnace |
  | Shell casing | brass ingot (makes several) |
  | Brass cartridge | shell casing + gunpowder + iron shot |

- **Casings come back:** firing a brass cartridge leaves a shell casing in the inventory, ready to reload. Paper cartridges burn up.
- **Military android ammo** (`add-ranged-combat`) becomes brass cartridges the player can use.

## Capabilities

### New Capabilities
- `firearms`: Guns, shooting in combat, the ammo crafting chain and reusable casings.

### Modified Capabilities
- `worldgen-depth`: nitre, sulfur and brass scrap deposits.
- `smelting`: brass nuggets smelt into brass ingots.

## Impact

- **Core:**
  - New items and recipes, with the reachability test covering them.
  - Three deposits in `world::DEPOSITS`, each with a mining drop.
  - A furnace smelting recipe for brass.
  - `MoveType::Shoot` in the player's moves, enabled by inventory.
  - Ammo use and casings returned.
- **Client:**
  - The Shoot move in the combat panel, with the gun's hit chance at the current distance.
  - Sprites for every new item and deposit.
- **Depends on:** `add-ranged-combat`, `overhaul-worldgen-depth`, and the crude furnace (later the electric furnace for the rifle's tier).

## Open question

- Should guns take a slot (an equipment or hotbar slot), or is "the best gun in the inventory" enough for now? This proposal assumes the latter.
