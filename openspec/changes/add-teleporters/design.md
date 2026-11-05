# Design

## Context

- **Power grid:** `PowerConsumer { demand_w, active, satisfaction_permille }`, integer joules per factory step.
- **Factory blocks and windows:** furnace and generator windows show the pattern.
- **Chunks:** generate lazily; `Game::factory_tile` generates a destination chunk with its structures.

## Goals / Non-Goals

**Goals:** fast travel between player-built points, gated by energy; deterministic.

**Goals (stations):** free travel between quest hubs the player has visited, starting with one in the bunker's wake-up room; a carried device for jumping to the network from anywhere.

**Non-Goals:** teleporting items or creatures, stations in random ruins, cross-world travel.

## Decisions

### Battery bank
- Capacity 50 MJ, charging at up to 200 kW while it isn't full: an active consumer whose demand is 200 kW when not full and 0 when full.
- Each factory step adds the delivered joules to `charge_j`. A short network charges it proportionally, like any consumer.

### Jump cost
`cost = 10 MJ + 5 kJ × distance in tiles`, where distance is Chebyshev in x/y plus 100 tiles per z-level of difference, capped at the capacity. The cost is checked and spent from the source teleporter only.

### Arrival
- The player lands on the first free walkable tile next to the destination, in a fixed order (north, east, south, west, then diagonals).
- The destination chunk is generated first, with its structures. If no tile is free, the jump is refused with a message.

### Names
- Default names are "Teleporter N", with N one higher than any existing number, so they're deterministic.
- Renaming uses a text field in the window.

### SapienCorp transit stations
- `FactoryKind::TransitStation`, placed by a `teleporter_station` structure block. A station can't be broken or picked up.
- `TransitStation { name, activated }`, saved. The name comes from its location ("Bunker Station", "Factory Station", "Android Station", "Facility Station").
- **First use:** interacting with a dormant station activates it and logs the handshake message. It does not jump on that same interaction. Later interactions open the destination list.
- **Placement:** in the bunker, the station goes in the west room with the player spawn (`!` in `shape_layers.txt`). The factory, the android's site and the arm quest's facility each get one near their entrance.

### Destinations by source
| Source | Lists | Cost |
|---|---|---|
| Transit station | other activated stations | free (SapienCorp reserve power) |
| Player teleporter | other teleporters and activated stations | the jump cost, from its battery bank |
| Mobile tele-access device | every teleporter and activated station | the jump cost, from carried power cells |

Dormant stations are never listed.

### Mobile tele-access device
- A gear item. Using it opens the destination window.
- The jump cost uses the formula above, from the player's position. It is drawn through `add-power-cells`' `use_power` helper: from the fullest carried cell, and refused with "no charge" (and the amount short) when the cells can't cover it.
- Sized so a standard cell covers one or two jumps: a late convenience, not a replacement for stations.
- The recipe needs scrap electronics and mid-game materials, with placeholders until those tiers exist.

## Risks / Trade-offs

- [Trivializes travel] → The cost scales with distance, and the battery needs a real grid.
- [Teleporting into danger] → The landing is next to the player's own block; combat starts normally if enemies are near.
- [Free stations make walking pointless] → Stations only reach other stations, at fixed quest hubs, and each must be reached on foot first to activate it.
- [The device waits on power cells] → Stations and player teleporters don't depend on `add-power-cells`, so they can ship first; the device task waits.
