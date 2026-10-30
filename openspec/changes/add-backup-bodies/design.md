# Design

## Context

- **Death (`add-player-death`):** a wreck, then `Game::respawn` with `Body::default()` at the bunker spawn.
- **Bodies:** `Body.parts` maps `BodyPartType` (Head, Torso, PowerSource, LeftArm, RightArm, LeftLeg, RightLeg) to parts with integrity and state.
- **Factory blocks** with slots and power exist (generators, machines, the furnace window).

## Goals / Non-Goals

**Goals:**
- Death costs preparation.
- Spare bodies are visible and choosable.
- The junker is a fallback that pushes the player to build a vat.
- The flyover is cosmetic and skippable.

**Non-Goals:**
- Upgraded or Enhanced parts (a later body-upgrades change).
- Transferring the inventory (it stays with the wreck, as now).
- Multiplayer.

## Decisions

### Tiers (decided by the user)
| Tier | When | Power | Parts it takes | Body layouts |
|---|---|---|---|---|
| Crude vat | starter | none | crude parts (wake Damaged, at 40 integrity) | the default two arms, two legs |
| Standard vat | mid-game | a trickle | crude or standard parts (standard wake Functional at 100) | the default layout |
| Advanced vat | end-game | a lot | any grade, advanced parts wake Enhanced | the default layout for now; four, six or eight arms come with `add-multi-arm-bodies` |

**Multi-arm layouts deferred (the user's decision):** bodies can't have more than two arms until `add-multi-arm-bodies`, so the advanced vat ships with the default layout, and that change gives it multi-arm torsos.

Recipes and power numbers are set in implementation and pinned by the reachability test and the power-balance examples.

### Parts and the vat
- Each part type gets an item per grade (crude, standard, advanced). A part's grade sets the state and integrity it wakes with.
- **The vat:** one slot per part in its layout. Its spare body is ready when the torso and the fusion core slots are filled (the rest can be missing, and the robot wakes without them) and, for powered tiers, it has power.
- **Power:** standard and advanced vats draw power (`demand_w` like other machines) while they hold a ready body.

### Resurrecting
- **The Resurrect screen** lists ready vats in a fixed order (by distance, then position), each with its parts' states. The junker option is always listed last.
- **Choosing a vat:** the robot is moved there, gets the spare body (parts from the slots, empty slots Missing) and the starting kit, and the vat's slots are emptied.
- **The junker:** every part Damaged at 30 integrity, at the SapienCorp HQ (the facility's entrance), with the thought line logged.

### Flyover
- A client-only camera path from the wreck to the destination, in a fixed duration (about 3 seconds of `app.clock`), easing zoom out, pan, and zoom in to level 3.
- The game state is resurrected when the choice is made. The flyover only draws, so skipping it changes nothing.

## Risks / Trade-offs

- [Too costly early] → The junker is always there, and the crude vat is cheap and needs no power.
- [The HQ is far from the bunker] → That's the point of the junker; the vat is the fix.
