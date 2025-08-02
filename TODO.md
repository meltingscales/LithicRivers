# old but still important TODO

- GH actions
  - can I unify the 4 release actions? hopefully
  - simplify files? :P
  - https://stackoverflow.com/questions/63148639/create-dependencies-between-jobs-in-github-actions
  - https://www.edwardthomson.com/blog/github_actions_17_dependent_jobs.html

## game
- add procedurally generated catgirls
- allow player to place blocks
- pushboxes
- audio system
  - music
- crafting
- hunger
- entities that move
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
- Steam build pipeline
- steam store page

## ux and polish
- save system with auto backups
- settings and options menu
- sound!
- tutorial and onboarding
- error handling
- performance...lmao its a TUI

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


# 2025 todo

- add a reference to hushy and lapfox Trax to the game somehow. music, sprites, items, tools, etc

- actually establish the game's lore

- modular body parts system like Qud, but more emphasis on crafting and modding

- start as an android with only 1 arm and 1 full leg, 1 damaged leg, with a quest to upgrade your body to less damaged. walk speed debuff, attack debuff, etc. "armless legless" as a funny challenge. 

- ability to program automation with simple block based logic, i.e. Redstone, and, computercraft-like programming system that uses Lua or similar scripting language

- EXP doesn't exist. rather, your stats are directly dictated by your equipment and body composition 

- relics and techmagic system

- heavy rewards for learning automation but not required to finish the game at all. you can choose to play it as a factory builder, or you can choose to play it as a hack and slash dungeon crawler.

- procedurally generated dungeons that ARE NOT just structures, but similar to how the original rogue generated dungeons

- a couple main questlines that reward different play styles

- options for true rogue like, "roguelite", and custom difficulty templates

- worldgen does not seem like it's actually seeding correctly, it seems to vary when new worlds are created with the same seed.

- add a "fog of war" feature that obscures unvisited sections of the world, and add a limit to player vision. previously-visited sections of the world should appear as grayscale.

- make sure the numpad keybindings also move the cursor for our asciimatics dialog boxes. try to do this in the cleanest way possible, so that asciimatics knows we're selecting a specific option.

- add multiple dimensions (like space, the void, etc) to the game.

- add the ability to save (or auto-save) to a world file. just use serialization now, but later, we should optimize it like how Minecraft does - chunks, dimensions, and a custom binary file format with auto-saving and multi-threaded loading/saving.

- add durability to crashes to the world saving process. maintain a WAL like sqlite perhaps? or just snapshot every 100 ticks?

- are there any really fun terminal characters we can use as an alternative to just ASCII? can you list them?

- add the ability to look around with 'L', just like caves of qud.  

- add an inventory screen as a tab on the bottom, like the help page and message log. make it really simple - just a simple list that you can page through with a cursor.

- add a "commands" screen as a tab on the bottom, that lets you perform macro actions like walking for 2,000 steps or walking to a specific coordinate.

- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI

- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI

- eventually, I'd like to publish this game on steam for $3. don't add/edit any code, I just want advice. What should I focus on before that?

- give me suggestions for different biomes. I'd like a "Lithic River" biome to be an underground biome with molten lava and lots of ores...like the name of the game.

- When you interact with an NPC -or- an Entity, I want you to be able to choose from a list of all NPCs AND entities that are adjacent to you. Currently, if you're next to 1 or more entities and at least 1 NPC, you immediately start dialogue with the NPC, but I'd rather it open a menu that lets you pick from a list of NPCs as well as Entities.