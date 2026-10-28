# Design

## Context

- **Quests:** `ActiveQuest { quest_type, state, objectives, rewards }`, with objectives `FetchItem`, `TalkToNPC` and `GoToLocation`. `QuestType` has only `RepairBrokenAndroid`.
- **The facility:** `Game::new` queues `sapiencorp-factory.lrstructure` at (200, 1609, 1), buried, and adds a quest marker. Structure tiles that spawn things are handled in `structure::spawn_entity_for_block` (treasure, scrap, enemies); `TreasureQuest1` isn't one of them.
- **Bodies:** `Body.parts` with integrity and state. `RepairRecipeHandler::can_repair_part` refuses Missing parts.

## Goals / Non-Goals

**Goals:** a quest that can always be finished; one arm per world; works across save and reload; deterministic.

**Non-Goals:**
- Other replacement parts (the install action is general, but only the arm item exists for now).
- Arm-dependent actions (`add-body-damage-effects` gives arms their gameplay).
- Redesigning the facility's levels beyond placing the chest.

## Decisions

### The chest
- `TreasureQuest1` spawns through `spawn_entity_for_block` as a factory `Chest` holding one `SalvagedArm`.
- `Resources::quest_chests_placed` (saved) records it, so re-placing the structure (re-entering its chunk) never adds a second arm.

### The quest
- When `RepairBrokenAndroid` completes, the android's next dialogue offers "A New Arm". Accepting adds the `ReplaceArm` quest, with objectives:
  1. `GoToLocation` (the facility's entrance, within 6 tiles)
  2. `FetchItem` (the arm, not consumed)
  3. `InstallPart { part: LeftArm }`
- Objectives complete in order and are checked each tick, like the tutorial's. Completing the last one sets the quest Completed and logs the android's closing line.

### Installing
- `Body::install(part, item) -> Result<(), InstallError>` only accepts a Missing part and an item that fits it (a table of part types per replacement item). It sets the part to Functional at 70 integrity.
- The Body tab shows "Install Salvaged SapienCorp Arm" on the missing arm when the item is in the inventory.

### Where the arm is
- In the assembly-line level, past its enemies, so reaching it is the quest's fight.
- `overhaul-worldgen-depth` changes the layers around the facility. Its entry must stay reachable from the surface; that change's checks include the facility.

## Risks / Trade-offs

- [The player installs the arm before the quest asks] → The install objective checks the arm's state, not the action, so it completes at once.
- [The arm chest is broken or emptied before the quest] → Breaking the chest spills the arm as usual. The fetch objective counts the item wherever it is in the inventory.
- [Old saves without the chest] → Saves from before this change have the facility placed already. When such a save loads with no chest recorded, the arm is placed once at the facility's chest position.
