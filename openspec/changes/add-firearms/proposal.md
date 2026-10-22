# Proposal

## Why

After `add-ranged-combat`, enemies can shoot, but the robot can only punch and tackle. The user's idea (IDEAS-INBOX.md) is guns with a real supply chain: gunpowder, paper, ammo, shell casings, and brass from nuggets to ingots. That turns fighting into a factory problem: ammo is something you make, and spent casings come back to be reloaded.

## What Changes

- **Two guns to start:**
  - a **pipe gun** (early, crafted from iron scrap and wood; fires paper cartridges; short range, poor accuracy)
  - a **rifle** (later, needs brass and the electric furnace tier; fires brass cartridges; long range, good accuracy)
- **Weapon slots** (decided by the user): every working arm holds one weapon. A gun is equipped into an arm's slot in the Body tab; a missing or destroyed arm can't hold one.
  - The starting robot, with its left arm missing, has one weapon slot.
  - Multi-arm builds (four, six or eight arms, from `add-multi-arm-bodies`) can wield that many guns.
- **Shooting in combat:** with at least one equipped gun and matching ammo, the combat move list gains "Shoot".
  - One Shoot move fires **every equipped, loaded gun in the same turn**, each rolling its own hit. A four-armed robot with four pipe guns fires four shots per Shoot.
  - Shots use the same line of sight, accuracy falloff and seeded hit roll as enemy shots (`add-ranged-combat`), with each gun's own range, accuracy and damage.
  - The move takes as long as the slowest gun fired.
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
  - Weapon slots on the body (one per working arm), saved with the body.
  - `MoveType::Shoot`, enabled by an equipped, loaded gun, firing every one in the same turn.
  - Ammo use and casings returned.
- **Client:**
  - Equipping guns into arm slots in the Body tab.
  - The Shoot move in the combat panel, listing each equipped gun with its hit chance at the current distance and its ammo.
  - Sprites for every new item and deposit.
- **Depends on:**
  - `add-ranged-combat`
  - `overhaul-worldgen-depth`
  - the crude furnace (later the electric furnace for the rifle's tier)
  - `add-multi-arm-bodies`, for more than two weapons

## Decided

- **Slots (the user's decision):** guns go in weapon slots, one per working arm, and multi-arm builds fire every equipped gun in one turn.
