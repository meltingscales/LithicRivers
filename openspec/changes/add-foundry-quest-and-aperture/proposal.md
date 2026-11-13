# Proposal

## Why

Quest 4 names the foundry that cast the aperture's ring, and the game still has no ending. The planned finale, `add-endgame-teleporter`, depends on materials from changes that aren't built (petrogen fuel, enriched uranium, strange matter), so as written it is far off. This change gives the game its ending now: a fifth quest that finally tells the player what they are for, and a biomechanical teleporter, the Aperture, built and fed from the endgame materials the game already has.

## What Changes

- **The SapienCorp Foundry:** a guaranteed surface ruin, one per world, far from the bunker. Its casting hall holds the aperture's casting patterns in a vault across a molten casting pit; the player bridges the pit with blocks (placing a plain block on lava leaves walkable crust). Guarded by military androids.
- **Quest 5, "The Foundry":** offered by SC-M4X7 once "Under the River of Fire" is complete.
  1. Reach the foundry.
  2. Cross the casting pit.
  3. Recover the aperture casting patterns.
  4. Bring them to SC-M4X7.
- **The reveal:** handing in the patterns reveals the Aperture. SC-M4X7 says the frames were made to open it again, and that the player is one of them. Until then, nothing in the Crafting tab, the Item Dictionary or the hotbar picker shows any of it.
- **The Aperture** (replaces `add-endgame-teleporter`'s scope):
  - An 11-by-11 multiblock: a core at the center, frame blocks around the border, and five input ports on its edges, each taking one kind of item.
  - **Inputs** the factory can make without end today: perfect biomech embryos (bio-vat), meat slurry (sheep farm), ethanol and mash (fermenter), and biochar (furnace), each at a required rate per game minute. Hand-mined materials (uranium, riverglass, plasteel) go into building it instead, since nothing mines them automatically yet.
  - **Charge:** every game minute, full rates add a point, a short input adds only its fraction, and a long shortage drains a point. 120 points (about two hours of game time at full rates) opens it.
  - **The flow window:** each input's required and actual rate over the last minute and ten minutes, colored green, yellow or red, plus the charge and an estimate of the time left.
  - **Recipes** for the core, frame blocks and ports, from advanced parts, riverglass, plasteel, reactor casing scrap and uranium: the rare materials are a one-time cost of building it.
- **The ending:** a full charge opens the Aperture: a closing sequence, then the credits, and the game goes on with the Aperture standing open. What lies past it (the far side) is a later change; for now, stepping in shows "The far side is not ready for you yet."
- **Performance:** a bench scenario with a large running factory feeding the Aperture, inside the tick budget.
- **Removed:** the `add-endgame-teleporter` change folder, now covered here. Later materials (petrogen, enriched uranium, strange matter) can join as extra inputs when their changes land.

## Capabilities

### New Capabilities
- `foundry-quest`: the foundry, its casting pit, the quest's steps and the reveal.
- `aperture`: the reveal gate, the multiblock, its inputs, charge, flow window and the ending.

### Modified Capabilities
None. The earlier quests' capabilities are in unarchived changes; this change only reads their completion.

## Impact

- **Core:**
  - `QuestType::Foundry`, its objectives, dialogue, hand-in and the reveal.
  - The foundry placed like the datacenter and annex.
  - `Resources::revealed` (saved) gating recipes, dictionary entries and placement.
  - Aperture blocks, multiblock detection, per-port rate tracking, the charge model, and the "opened" state.
  - New items: the casting patterns, the Aperture core, frame and port.
- **Content:** the `sapiencorp-foundry` plan, checked with `just util-structure-preview`.
- **Client:** the flow window, the closing sequence and credits, sprites and 11-by-11 frame art, `tp_quest foundry`, and a `reveal aperture` cheat for testing.
- **Depends on:** quests 2 to 4 and their structures, lava crust bridging, the bio-vat (perfect embryos), the sheep farm (meat slurry), the fermenter (ethanol, mash), furnaces (biochar), reactors (uranium), all merged.
