# old but still important TODO

- GH actions
  - can I unify the 4 release actions? hopefully
  - simplify files? :P
  - https://stackoverflow.com/questions/63148639/create-dependencies-between-jobs-in-github-actions
  - https://www.edwardthomson.com/blog/github_actions_17_dependent_jobs.html

## game
- add procedurally generated catgirls
- auto-scale the viewport based off of viewable area
    - determine from screen size
    - determine from "blank space" in asciimatics (if this is even doable)
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


# 2025 todo

- add a "fog of war" feature that obscures unvisited sections of the world, and add a limit to player vision. previously-visited sections of the world should appear as grayscale.
- add multiple dimensions (like space, the void, etc) to the game.
- add the ability to save (or auto-save) to a world file. just use serialization now, but later, we should optimize it like how Minecraft does - chunks, dimensions, and a custom binary file format with auto-saving and multi-threaded loading/saving.
- instead of just "World Gaia", I want you to add some useful GUI elements to be rendered at the top of the screen with a small border. Things like health, stamina, current heading, etc. Make sure it's left-aligned but also takes up as much space as is possible (same width as world viewport)
- are there any really fun terminal characters we can use as an alternative to just ASCII? can you list them?
- make the message log pane actually functional - dialog, interactions, mining, and pickups should be logged there.
- add the ability to look around with 'L', just like caves of qud.
- add an inventory screen as a tab on the bottom, like the help page and message log. make it really simple - just a simple list that you can page through with a cursor.
- add a "commands" screen as a tab on the bottom, that lets you perform macro actions like walking for 2,000 steps or walking to a specific coordinate.
- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI
- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI
 - eventually, I'd like to publish this game on steam for $3. don't add/edit any code, I just want advice. What should I focus on before that?
- give me suggestions for different biomes. I'd like a "Lithic River" biome to be an underground biome with molten lava and lots of ores...like the name of the game.


- add random but seeded structure generation - old ruins, ore patches, machines, crashed ships, behemoth corpses, etc. Have the structure definition files be stored in ./lithicrivers/data/structures/ and come up with a reasonable format for them that lets you use ASCII art to define structures. TILDE in shape_layers.txt is a special character that separates a y-layer. for example:

/lithicrivers/data/structures/small_ship.lrstructure/
/lithicrivers/data/structures/small_ship.lrstructure/shape_layers.txt

....
.XX.
....
~~~~
.XXX
Xe A
XXX.
~~~~
....
.XX.
....


/lithicrivers/data/structures/small_ship.lrstructure/data.json
{
  "blocks": {
    ".": "empty",
    "X": "iron_scrap",
    "A": "door",
    "e": "scrap_electronics",
  },
  "gen_biomes": "ALL",
  "gen_chance": 0.001,
  "y_layer_gen_range": [0, 0] //can generate only on layer 0, the ground.
}

I'd like you to also add a giant corpse (bone blocks), and a small temple (iron scrap with a buried treasure 2 blocks below the center)