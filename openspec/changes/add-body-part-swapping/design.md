# Design

## Context

- `Body::install(part, item)` fills a missing part at `INSTALLED_PART_INTEGRITY` (70). `replacement_parts(item)` lists the parts an item fits; only `SalvagedArm` exists today.
- `Game::install_part(item)` picks the first missing part that fits; the Body tab's `[i]` calls it. The repair window has its own part selection.

## Goals / Non-Goals

**Goals:** choose the part; install, swap and remove; no free repairs; works with keys and mouse; general enough for the parts `add-backup-bodies` and `add-multi-arm-bodies` add.

**Non-Goals:** new part items or grades (`add-backup-bodies`); extra limb slots (`add-multi-arm-bodies`); part-specific stats beyond today's effects.

## Decisions

### What a removed part becomes
- Items carry no integrity, so a removed part can't keep its wear. A Functional or Enhanced part returns as its item (reinstalling gives 70, never more than it had above Damaged). A Damaged part returns as scrap: Iron Scrap and Scrap Electronics, one each.
- Parts without an item (today: everything but arms) come out as that scrap too, until `add-backup-bodies` gives them items.

### Locked parts
The torso and power source can't be removed; the remove key says why.

### Selection
- The Body tab keeps `selected_part`. Up and Down move it, a click selects it, and the fitting-items list follows it.
- With items listed, Left and Right move between the part list and the items, as in item windows.
- Opening repair (`R`) preselects the selected part.

### Game logic in core
`Game::install_part_into(part, item)` and `Game::remove_part(part)` own the rules and the messages, so the client only selects and calls them.

## Risks / Trade-offs

- [Removing a working part by accident] → Removing needs the key while that part is selected, and the item goes to the inventory (or the floor when full), so it can be reinstalled.
- [The arm quest's install objective] → It checks the arm's state, so installing through the new list still completes it.
