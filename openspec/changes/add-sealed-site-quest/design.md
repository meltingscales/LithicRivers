# Design

## Context

- **Quests 2 and 3** are the pattern: a guaranteed structure queued as a `PendingStructure` (`Game::new`, and `Game::queue_datacenter` for old saves), ordered objectives checked each tick (`quests::advance`, including `PoweredAt` for sealed doors), dialogue nodes on the repaired android gated by `requires_quest_complete`, and a core hand-in function the client calls after the dialogue choice takes the item (`quests::finish_black_box`).
- **Sealed doors** (`sealed_door.rs`) open while their network has power. Structure tiles become built-in blocks (`sealed_door_site`, `burner_generator_site`, `power_pole`, station tiles), and quest chests carry their item (`TreasureQuest1` the arm, `TreasureQuest2` the data shard).
- **Buried structures** put their top layer at the placement z and go down; the facility is buried eight layers. A structure spanning several z-chunks marks them all generated (`mark_all_structure_chunks_generated`).
- **Mining** costs one action of `BodyEffects::mine_cost(300)` ticks, whatever the tile (`Game::mine_cost`). Drops come from `crafting::mining_drop`.
- **Power:** generators supply their network each factory step; burners up to 90 kW. Machines with an `Inventory` get the two-panel machine window through `OpenMachine`.
- **`NuclearWaste` already exists as an item** (nuclear slimes carry it); this change adds a tile that drops it.
- **`add-ore-spouts` isn't built**, so there are no iron ingots.

## Goals / Non-Goals

**Goals:**
- A fourth quest that follows quest 3's hook, teaches the thermoelectric generator by making the player build one to get in, and reveals the aperture.
- The reactor tier from `add-ruined-reactors`, adjusted to what exists today.

**Non-Goals:**
- Radiation damage, meltdowns, player-built reactors and uranium refining (still later tech-tree changes).
- Quest 5 or its marker.
- Explaining the player's purpose; the endgame stays secret.

## Decisions

### One structure for shaft and annex
`thale-annex` is a buried plan of 13 layers: the top layer (z = 0) is a small SapienCorp access hut on the surface, layers 11 to 1 are rock with a 3x3 shaft and a column of stairs, and layer 0 (z = -12, in the deep band) is the annex. It is queued at `ANNEX_POS` = (-260, -820, 0), far from the bunker, the facility and the datacenter. Alternative: rely on natural stairways and place only the annex. Rejected: a guaranteed quest needs a guaranteed way down.

"Under the river of fire" is built into the plan: a lava channel crosses the annex's arrival hall with one crust ford, so the annex always sits under a river whatever worldgen does around it.

### The annex layout (layer 0)
- **Arrival hall:** the bottom of the stairs, the lava channel and its ford, and the dormant Annex Station with its RTG, charger and pole, kept more than a wire's reach from the door's pole.
- **Outer reactor room:** a cracked containment ring of reactor casing (at least four tiles), three nuclear waste blocks, iron scrap and scrap electronics: enough to craft one thermoelectric generator and fuel it. The blast door's pole stands here, with open floor beside it for the generator.
- **Thale's lab,** behind the blast door (a `sealed_door_site`): a `TreasureQuest3` chest with Thale's journal, rare scrap and plasteel, and one military android.

The door's network has a pole and nothing else, so only a generator the player places powers it.

### Ruined reactors
A separate roll per surface chunk (`cz == 0`, not city), from its own feature generator so the existing structure roll is untouched: 1.5% chance, then one of three layouts, buried so a layer sits below ground:

| Layout | Size | Uranium | Waste |
|---|---|---|---|
| Cracked containment ring | 11 x 11 | 1 | 6 |
| Half-buried core | 9 x 9, 2 layers | 2 | 8 |
| Spent-fuel pond | 7 x 7 | 1 | 5 |

### Tiles, drops and hardness
| Tile | Breaks into | Drop | Hardness |
|---|---|---|---|
| ReactorCasing | Air | 1 reactor casing scrap | 5 |
| NuclearWasteDeposit | Air | 1 nuclear waste | 1 |
| UraniumDeposit | Air | 1 uranium chunk | 1 |

`MiningDrop` gains `hardness` (1 for every existing tile). The mine action reads the target tile when it is queued and multiplies its cost by the hardness.

### Thermoelectric generator
`FactoryKind::Thermoelectric` with an `Inventory` of one slot that takes only nuclear waste or a uranium chunk, so the existing machine window and belts work with it. Each factory step it supplies 30 kW (waste) or 60 kW (uranium) to its network, and nothing when empty; the item is never used up. Recipe: 4 iron scrap, 2 scrap electronics, 2 reactor casing scrap. Alternative: a dedicated heat-source component and window. Rejected: an inventory slot gives the window and belt handling for free.

### The quest
`QuestType::SealedSite`, "Under the River of Fire", offered from the repaired greeting once "Black Box" is complete:

1. "Reach Thale's Annex" (`GoToLocation`, the bottom of the shaft)
2. "Power the annex door" (`PoweredAt`, the blast door)
3. "Recover Thale's journal" (`FetchItem`, not consumed)
4. "Bring the journal to SC-M4X7" (`TalkToNPC`, completed by the hand-in choice)

The hand-in follows quest 3: the choice requires the journal, the generic dialogue code takes it, and the client calls `quests::finish_sealed_site`, which completes the quest and gives an advanced arm and a uranium chunk. `quest_chests_placed` gains a third chest; `tp_quest locker` already picks the one nearest the facility, and `tp_quest annex` goes to the bottom of the shaft.

### Lore
- **The journal** (item description and the reveal dialogue):
  - The aperture was Thale's gate to "the far side," fed by the reactor held past its limit.
  - It opened once, for four seconds: "We heard the far side breathing. It turned toward us."
  - Thale ordered the maintenance frames built to "remember the aperture's shape, in case we do not."
- **The hook for quest 5:** the journal's last page names the foundry where the aperture's ring was cast.
- **SC-M4X7's reaction:** it goes quiet, then says the frames were its own model line.

None of it says what the player is for.

## Risks / Trade-offs

- [The door puzzle stalls players who don't know the recipe] → SC-M4X7's offer says the annex runs on its own reactor's leavings; the Crafting tab lists the thermoelectric generator once its scrap is carried, and the annex holds every ingredient.
- [A 13-layer structure is a big plan] → Shaft layers are one fill and a stairs shape each; the lint checks the stairs column.
- [Hardness touches every mine action] → Defaults to 1; only reactor casing changes, with a test for both.
- [Reactor ruins collide with other structures] → They roll after the existing structure roll and skip chunks where it placed something.
- [Old saves] → The annex is queued on load like the datacenter, only in worlds with the quest line.

## Migration Plan

New worlds get the annex in `Game::new`; old saves get it queued on load. Ruins appear only in chunks generated after the update. `add-ruined-reactors` is removed in this change's PR, since this change covers it.
