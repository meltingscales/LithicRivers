# Design

## Context

- **Player combat moves** (`moves.rs`): Melee, Tackle, Escape and others, each with energy, time and damage.
- **`add-ranged-combat`:** `line_of_sight`, the hit chance falloff, and `CombatAction::EnemyShoot`.
- **Deposits:** `world::DEPOSITS` (vein noise per band). The crude furnace smelts by recipe (`furnace::smelting_recipe`).
- **Crafting:** every placeable item must be craftable from raw sources (`crafting` tests).

## Goals / Non-Goals

**Goals:**
- Ammo is a production line.
- Casings loop back.
- Guns use the same rules as enemy shots.
- Deterministic.

**Non-Goals:**
- Armor and other equipment slots (only weapons, one per arm).
- Gun upgrades and mods.
- Explosives.
- Outside-combat shooting.
- Copper and zinc spouts. Brass comes from salvage until `add-ore-spouts` adds copper.

## Decisions

### Weapon slots (decided by the user)
- Each arm part has a weapon slot holding at most one gun item. Equipping moves the gun from the inventory into the slot, and unequipping moves it back.
- An arm that is Missing loses its slot: its gun drops into the inventory, or onto the ground if the inventory is full.
- The slots are saved with the body. A wreck (`add-player-death`) keeps the guns in its body's slots.

### Shooting
- `MoveType::Shoot` is offered when at least one equipped gun has ammo.
- One Shoot fires every equipped gun that has ammo, in a fixed order (arm order). Each rolls its own hit and uses its own cartridge, at the same target.
- The move queues like others and takes the slowest fired gun's time.
- A Damaged arm's gun shoots at 75% of its accuracy, matching `add-body-damage-effects`.
- **Hit chance:** the same falloff formula as enemy shots, with each gun's numbers:

  | Gun | Range | Accuracy at 1 tile | Falloff per tile | Damage | Time |
  |---|---|---|---|---|---|
  | Pipe gun | 6 | 70% | 8% | 10–18 | 12 ticks |
  | Rifle | 12 | 90% | 4% | 16–26 | 10 ticks |

- A shot uses one cartridge. A brass cartridge adds one shell casing to the inventory.

### Materials
- **Deposits:**

  | Deposit | Band | Share (pinned by the survey test) |
  |---|---|---|
  | Nitre | shallows | about 0.3% |
  | Sulfur | deep | about 0.3% |
  | Brass scrap | deep | about 0.2% |

  Each drops its item when mined.
- **Smelting:** 4 brass nuggets make 1 brass ingot in the crude furnace (a new smelting recipe).
- **Recipes:**

  | Makes | From |
  |---|---|
  | 3 shell casings | 1 brass ingot |
  | 2 gunpowder | 1 biochar + 1 nitre + 1 sulfur |
  | 2 paper | 2 wooden shavings |
  | 4 iron shot | 1 iron scrap |
  | 2 cartridges of either kind | the parts in the proposal |

  Guns come from iron scrap, wood and, for the rifle, brass ingots and scrap electronics.

## Risks / Trade-offs

- [Guns trivialize combat] → Ammo costs a chain of materials, the pipe gun is inaccurate, and enemies scale later.
- [Many new items] → All go through the Item Dictionary, which shows where each comes from and what uses it.

## Implementation notes

- **Slots** live on the body as `Body::weapons` (arm to gun), saved with it. `firearms::ARMS` lists the arms that fire, in order; `add-multi-arm-bodies` extends it.
- **Equipping:** the Body tab's Wield key (`body.WIELD`, `w`) puts the best carried gun (the rifle before the pipe gun) in the selected arm, swapping out what it held, or puts the gun away when no other is carried.
- **Shoot** is listed after Melee only while a gun has rounds; two guns of one kind need a round each. The selected enemy's card shows each gun's chance from there and the rounds carried.
- **Shared rules:** player and enemy shots both go through `systems::ranged::fire` (range, line of sight, seeded roll, the shot's line and sound).
- **Deposit shares** for seed 12345: nitre 0.2 to 0.6% by shallow layer, sulfur about 0.3% and brass scrap about 0.2% in the deep. Military androids also carry 1 to 2 brass nuggets.
- **Recipes:** pipe gun from 3 iron scrap, 2 planks and 4 nails; rifle from 2 brass ingots, 3 iron scrap, 2 scrap electronics and 1 plasteel scrap (plasteel comes from the deep).
