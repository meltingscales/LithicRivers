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


# eventually todo

- plant growth that's self limiting and based on nearby blocks, but allows for farming manually or by automated means

- interface for blind people to play the game, modular command lookup and help system. actually test it with popular screen readers.

- a world map system like Qud that allows fast travel, and make the map tiles an average of the cells in a current chunk: a nuked chunk actually looks like a wasteland on the global map

- restrict the ability to travel in the Z axis to stairs blocks only, unless you're flying or have a giant drill

- make sure that building is not annoying! it should be really fun and easy.

- hot bar system like Factorio with multiple swappable items and easy editing

- ability to toggle variable Z-axis rendering for tall buildings

- separate 3D world explorer program, a la mcedit 

- separate web portal for viewing game stats, map, entities, and performance 

- add a reference to hushy and lapfox Trax to the game somehow. music, sprites, items, tools, etc

- actually establish the game's lore: the world ends after a fusion reactor explodes in the year 2186. this triggers a runaway greenhouse effect in the Earth's atmosphere that kills most humans and causes the world's economy to devolve. most complicated manufacturing gets taken over and destroyed. fast forward 1,000 years later to the year 3186, and you awaken as a heavily damaged android from SapienCorp, an old company that used to produce AI robots. you only wake up due to a 32-bit date overflow error that causes you to turn on and awaken in a desolate wasteland of a world with no real goals other than to repair yourself and explore.

- modular body parts system like Qud, but more emphasis on crafting and modding

- fuel cells of different types, jetpacks, antigravity (with extremely comedic "fall into space" mishaps), falling mechanic, electrical tools, gas powered tools

- make inventory management not annoying: simple controls, toggle selections, "are you sure" to destructive or dangerous actions, "junk" rules or auto-dump option

- start as an android with only 1 arm and 1 full leg, 1 damaged leg, with a quest to upgrade your body to less damaged. walk speed debuff, attack debuff, etc. "armless legless" as a funny challenge. 

- ability to program automation with simple block based logic, i.e. Redstone, and, computercraft-like programming system that uses Lua or similar scripting language

- EXP doesn't exist. rather, your stats are directly dictated by your equipment and body composition 

- relics and techmagic system

- heavy rewards for learning automation but not required to finish the game at all. you can choose to play it as a factory builder, or you can choose to play it as a hack and slash dungeon crawler.

- procedurally generated dungeons that ARE NOT just structures, but similar to how the original rogue generated dungeons

- a couple main questlines that reward different play styles

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

- add the ability to look around with 'L', just like caves of qud.  

- add an inventory screen as a tab on the bottom, like the help page and message log. make it really simple - just a simple list that you can page through with a cursor.

- add a "commands" screen as a tab on the bottom, that lets you perform macro actions like walking for 2,000 steps or walking to a specific coordinate. probably integrate this with the blind feature.

- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI

- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI

- eventually, I'd like to publish this game on steam for $3. don't add/edit any code, I just want advice. What should I focus on before that?

- give me suggestions for different biomes. I'd like a "Lithic River" biome to be an underground biome with molten lava and lots of ores...like the name of the game.

## MVP for steam release (2026)

- perlin noise worldgen
- block placement, push boxes, and fluids
- basic crafting and a body repair/modular body feature with a damaged android body
- procedural dungeons
- body modularity means dynamic walk and break speeds, so tick rate needs to be larger than 1, perhaps 200 or so.



## steam key giveaway list:
skomor123