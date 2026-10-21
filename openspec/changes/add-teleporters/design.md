# Design

## Context

- **Power grid:** `PowerConsumer { demand_w, active, satisfaction_permille }`, integer joules per factory step.
- **Factory blocks and windows:** furnace and generator windows show the pattern.
- **Chunks:** generate lazily; `Game::factory_tile` generates a destination chunk with its structures.

## Goals / Non-Goals

**Goals:** fast travel between player-built points, gated by energy; deterministic.

**Non-Goals:** teleporting items or creatures, world-generated teleporters (maybe as ruins later), cross-world travel.

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

## Risks / Trade-offs

- [Trivializes travel] → The cost scales with distance, and the battery needs a real grid.
- [Teleporting into danger] → The landing is next to the player's own block; combat starts normally if enemies are near.
