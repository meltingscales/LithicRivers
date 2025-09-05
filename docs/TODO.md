# LithicRivers Development Priorities

## Steam MVP (Target: December 2024)
See [STEAM-MVP.md](STEAM-MVP.md) for the complete release plan.

## game
- add procedurally generated catgirls
- allow player to place blocks
- pushboxes
- fighting
- explosions/fire
- perlin noise worldgen (like minecraft)
- conveyor belts
- fluids
- equipment
- growing crops
- mods
- space travel
- NPCs
- quests?
- stats?
- cybernetics/augs

# pre-steam-release TODO:

- steam SDK integration
- achievements system
- steam App ID
- steam store page

## ux and polish
- save system with auto backups
- tutorial and onboarding

## content
- core gameplay loop
- progression system
- content volume: At least a few hours of content. we should strive for proc-gen stuff because theres more replayability
- modding system??
- replayability

## steam-specific features
- achievements
- trading cardsp
- cloud saves
- controller support
- steam workshop

## marketing
- store page stuff
- community hub
- presskit

## legal
- ToS
- privacy policy
- age rating
- tax info

## QA
- bug testing
- perf testing
- user testing
- steam deck compat

## Priority Order for Your Project:
- Complete the save system (already in your TODO)
- Add Steam SDK integration
- Polish the core gameplay loop
- Add achievements system
- Create compelling store page content
- Implement cloud saves
- Add controller support
- Set up Steam Workshop for mods

# eventually todo

- print logo briefly when game boots (config/logo.txt)
- print boot log briefly when game boots (config/boot_message.dat), making sure to add corruption. Make sure to print it slowly. Create a custom UI element to do this. Also, add a "SKIP_INTRO" env var to skip this for development purposes.

- plant growth that's self limiting and based on nearby blocks, but allows for farming manually or by automated means

- interface for blind people to play the game, modular command lookup and help system. actually test it with popular screen readers. Make sure that it can be set via a command-line flag LITHICRIVERS_BLIND_MODE=1

- a world map system like Qud that allows fast travel, and make the map tiles an average of the cells in a current chunk: a nuked chunk actually looks like a wasteland on the global map

- restrict the ability to travel in the Z axis to stairs blocks only, unless you're flying or have a giant drill

- make sure that building is not annoying! it should be really fun and easy.

- hot bar system like Factorio with multiple swappable items and easy editing

- ability to toggle variable Z-axis rendering for tall buildings

- separate 3D world explorer program, a la mcedit 

- separate web portal for viewing game stats, map, entities, and performance 

- add a reference to hushy and lapfox Trax to the game somehow. music, sprites, items, tools, etc

- modular body parts system like Qud, but more emphasis on crafting and modding

- fuel cells of different types, jetpacks, antigravity (with extremely comedic "fall into space" mishaps), falling mechanic, electrical tools, gas powered tools

- make inventory management not annoying: simple controls, toggle selections, "are you sure" to destructive or dangerous actions, "junk" rules or auto-dump option

- start as an android with only 1 arm and 1 full leg, 1 damaged leg, with a quest to upgrade your body to less damaged. walk speed debuff, attack debuff, etc. "armless legless" as a funny challenge. 

- ability to program automation with simple block based logic, i.e. Redstone, and, computercraft-like programming system that uses Lua or similar scripting language

- EXP doesn't exist. rather, your stats are directly dictated by your equipment and body composition 

- relics and techmagic system

- heavy rewards for learning automation but not required to finish the game at all. you can choose to pla- make sure that Y less than 0 generates unique terrain, currently it seems static. maybe Y isn't getting used in the perlin noise gen?

- 10 to 20 static, main questlines NPCs, buildings, items, and dungeons

- "everything is craftable" and "everything is raw materials"

- simple query-based crafting system with a missing materials tracker like Factorio and the ability to view ratios for spreadsheet optimization 

- NPC tribes that naturally gather from the environment and build machines by themselves

- options for true rogue like, "roguelite", and custom difficulty templates

- worldgen does not seem like it's actually seeding correctly, it seems to vary when new worlds are created with the same seed.

- add a "fog of war" feature that obscures unvisited sections of the world, and add a limit to player vision. previously-visited sections of the world should appear as grayscale.

- make sure the numpad keybindings also move the cursor for our asciimatics dialog boxes. try to do this in the cleanest way possible, so that asciimatics knows we're selecting a specific option.

- add multiple dimensions (like space, the void, etc) to the game.

- add the ability to save (or auto-save) to a world file. just use serialization now, but later, we should optimize it like how Minecraft does - chunks, dimensions, and a custom binary file format with auto-saving and multi-threaded loading/saving.

- add durability to crashes to the world saving process. maintain a WAL like sqlite perhaps? or just snapshot every 100 ticks?

- are there any really fun terminal characters we can use as an alternative to just ASCII? can you list them?

- add a "commands" screen as a tab on the bottom, that lets you perform macro actions like walking for 2,000 steps or walking to a specific coordinate. probably integrate this with the blind feature.

- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI

- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI

- eventually, I'd like to publish this game on steam for $3. don't add/edit any code, I just want advice. What should I focus on before that?

- give me suggestions for different biomes. I'd like a "Lithic River" biome to be an underground biome with molten lava and lots of ores...like the name of the game.

- john feedback: what about gaussian noise (idk) **Gaussian Noise for Resource Distribution**: Implement Gaussian noise for ore deposits, cave systems, and rare resources. Unlike Perlin noise which creates smooth patterns, Gaussian noise creates scattered, realistic resource distribution. Use for: ore deposits (Gold, Iron), cave entrances, rare artifacts, atmospheric effects (acid rain, radiation storms). Combine with existing Perlin noise - Perlin for terrain/biomes, Gaussian for resources/events.

- GPU-accelerated worldgen: Option to use GPU for Perlin noise/worldgen calculations while maintaining determinism (compute shaders with fixed-point math or controlled floating-point precision)

- investigate github repo "gale93/sbixel"...

- investigate how dwarf fortress does vertical combat...

- based on the value of `world_seed % 1` will randomly spawn a friendly Greyhound dog named either Kopi or FortyNiner that follows you around and attacks anything that attacks you. If you finish the game with the dog alive and in  your party, you get a free copy of the game to give to a friend, as well as some poetry about my early childhood and my thoughts about my favorite music artists like lapfox trax and 4lung.

- fight system that is basically just chrono trigger, limit to 3 enemies so players dont get fucked
  - if you decide to "Escape", there's a "BattleDelay" timer that prevents you from re-engaging with an enemy for about 2000 ticks. This trait is stored on the entity itself and gets counted down when worldtick happens (as part of a BattleDelayTimer system)
  - Tackle is a move
    - It has a 800 tick cooldown
    - It ALWAYS pushes an enemy back 2 spaces
    - it stops combat if no enemies are adjacent to you now
    - You move into the enemy's original space
    - It 50% of the time will make an enemy Stunned for 600 ticks

- fights only occur on the same Z-level, no 3d combat. just like qud.

## MVP for steam release (2025 christmas release)
- Auto harvest Mode
- Dig mode (to not get stuck in rocks when moving Z axis)
- consider using cardboard/paper cutouts to model the game system
  - inventory pages
  - ui
  - fighting
    - dogs get attracted to food/organic matter

- sandboxed python programming: simple interface
  - examples of dead robots with sample programs, i.e.
    - TreeCutter.py
    - StripMiner.py
    - Sentry.py

- waypoint and map system

- "backup body" system for continuing after death
  - ...or using portable nukes without consequences...

- reintegrate speedscope for ratatui and rust build

- tutorial system that can be re-activated
  - context-aware tutorial
- game checkpointing, branching saves
  - "This feels very useful. You can access this memory any time in XYZ menu."
  - `git log --graph` style view
- john: What are the challenges or hazards?
  - puzzles
  - exploration
    - specialty bodies with:
      - more limb slots
      - more attachments
      - jetpacks (fuel, electric)
  - hazards
  - grappling hooks/shooting
- in progress: visualize body parts in the body panel with a custom asciimatics class that extends Frame...
- in progress: UI elements that let you select and repair body parts, and backend code that actually repairs the body part for a material cost.
- 🔄 **IN PROGRESS**: Block placement with body-based restrictions
- 🔄 **IN PROGRESS**: Push boxes that require certain body parts
- 🔄 **IN PROGRESS**: Basic crafting system for body repairs
- 🔄 **IN PROGRESS**: Procedural dungeons v2: Overhaul this and just copy the algorithm from https://www.gamedeveloper.com/programming/procedural-dungeon-generation-algorithm
- finish working on `story.txt`...
- in progress: steam API integration
- ask AI: How should I start doing steamworks/steamapi integration?

- save/load system where you can pick a save file from a list of saved games (build UI for this...)

## steam key giveaway list:
- skomor123
- noahnogueras@gmail.com
- oglingling
- college friends
- miguel from colab
- cameron (false)

## streamers

- caseoh
