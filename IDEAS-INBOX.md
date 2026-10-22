<!-- AI don't remove these beginning lines please -->

# Ideas inbox

## Start

Put new feats here :) 

Ask for "Add an openspec feat that adds XYZ" or "Bugfix: Something not rendering" or "Feat: Add sheep grass eating behavior"

## TODO ITEMS

<!-- AI feel free to edit below -->

<!-- MY THOUGHTS -->

.

- feat: special endgame "Fractal Tear" biome that is fractal-inspired, with mandelbrot/menger sponge/douady rabbit/bifurcation curve inspired worldgen for resources and blocks
  - most blocks can be processed with complex resource chains involving petrogen distillation chains (inspired by minecraft create mod diesel/oil) or complex uranium ore processing (inspired by GregTech or Factorio mods)
  - or just plain ore washing with extremely rare chances to yield useful materials
  - highly inspired by Moon Stair from Caves of Qud

- feat: different types of crops, wheat (to be used to make biofuel or ethanol), potatoes (same, perhaps different ratio of byproducts), carrots (can be used to breed certain animals)
  - auto-feeders that can be used to automate animal farms
  - animal counters that output a SapienNet signal based on how many animals are in a X-by-X radius of them
  - SapienNet network feature: cables (like power cables) that can send signals on a network, similar to redstone or ComputerCraft network cables
    - feat: some sort of unified logic editor that lets you conditionally trigger machine behavior based on logical operators, like in Factorio's circuit network
    - feat: lightweight Lua IDE that lets you implement more complex SapienNet logic, ideally with an IDE that's external-IDE-friendly (stores scripts in save folder but separate from gzipped json, for easy editing) and has some sort of guide or documentation, complete with examples of different machines. inspired by ComputerCraft.
  - some lore-friendly block that lets you damage non-player-entities for the purpose of farming
  - bio-vats that...do something with meat, not sure yet. To give a purpose to animal farm factories. Possibly require growing "{Deformed/Wrinkly/Perfect} Biomechanical Embryo" items for some crafting purpose.

- feat: biome compass: moderately expensive item that requires power from a power cell item, can be used to locate the closest "X type of Biome" and if activated, displays a waypoint on your Global Map

- feat: power cell items (with 3 tiers) that can be used to store/draw power from battery blocks (on electric grid) that then are used to feed power into items or gear (and get passively charged by your fusion core, depending on what tier it is)

- feat: endgame purpose is actually to build a strange biomechanical teleporter, not a rocket ship. The teleporter requires a continuous drip input of endgame items, forcing the player to build a factory capable of producing them for a sustained period of time. It should be a multiblock structure, 11x11 blocks wide, and a "item flow" GUI showing the player what's green (more than enough), yellow (sustained input/borderline), and red (falling) so they can plan their factory
  - feat2: possibly add a "Production over time" GUI like Factorio so players can see their factory's crafting chain insufficiencies
  - feat3: might need to add chunkloading or partial chunkloading for machine block interactions for this to work for large factories.
  - feat4: lore/questing: This should be completely unknown to the player until they complete a lot of quests. [AI: Put this in our design decisions/fundamentals: I want the player to uncover lore and quests slowly, slowly learning what actually happened to cause the SapienCorp reactor explosion that caused a nuclear holocaust)
- feat: enemies should respawn in some way, not sure how yet. perhaps environmental factors could spawn them (or literally rip off minecraft and make spawner blocks) - i.e. underground nuclear waste rivers=nuclear slimes sometimes materialize, or redwood forest biomes will spawn dogs, also need to put limits so we don't get floods of 100 enemeies or something.
  - enemy farming should be encouraged somehow
  - new block: fence: can see through but enemies can't pass through it

- feat: enemy that shoots projectiles at you, perhaps a rusty military android with limited ammo

- feat: guns, shooting mechanic, accuracy/dropoff, gunpowder, paper, ammo, shell casings, brass ingots, brass nuggets, etc etc

- feat: dying should create a death marker/waypoint that you can optionally delete with an ARE YOU SURE?

- feat: /keepinventory [on/off] cheat
- feat: /keepbody [on/off] cheat
  - dying and respawning gives you same limbs but fully repaired

- thought: a more AI-friendly way of generating multiblock structures since VoxelBuilder (see justfile) is a gui for humans. ideas, ai? it'd be nice to noodle with prompts instead of manually editing voxelbuilder 3d models then mapping colors to blocks/spawnpoints...

- feat: backup bodies can be crafted and will be transferred to upon death (with limb status, power level, etc shown on the "Resurrect" screen). if one or more is active (you get to choose where you respawn if there are >1), with limb/torso/energy core/legs/arms/etc being craftable too, and assignable to a "Robot Resurrection Vat". if you die without one, you respawn...well, I'm not sure yet. Maybe a crappy junker body at the SapienCorp HQ with a though bubble of "I should really make a vat..."?
  - feat2: cool respawn animation that plays after choosing a resurrection method (junker or vat) that takes your viewport through the overworld like you're flying, towards the respawn point, like in GTA where the camera zooms in, then out, then flies to the respawn point, then zooms in, then you respawn (with zoom level 3)

<!-- ACTIONED ON ALREADY -->
