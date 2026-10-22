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
- Equipment slots.
- Gun upgrades and mods.
- Explosives.
- Outside-combat shooting.
- Copper and zinc spouts. Brass comes from salvage until `add-ore-spouts` adds copper.

## Decisions

### Shooting
- `MoveType::Shoot` is offered when the inventory has a gun and its ammo. The best gun (the rifle over the pipe gun) is used.
- It queues like other moves, with each gun's time.
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
