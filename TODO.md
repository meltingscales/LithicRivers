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

- add a dialog box/popup system, as well as a starter NPC that you can talk to and choose a dialog option at the end of a short conversation. Also, add a keybind to directionally interact with an adjacent item/block/entity, just like Caves of Qud. When you press the "Interact" button, it should let you choose from a list of adjacent things via a popup. Test this by making 2 identical entities, adjacent to eachother, with different names and different colors.
- make the message log pane actually functional - dialog, interactions, mining, and pickups should be logged there.
- add the ability to look around with 'L', just like caves of qud.
- make the help page give a full keybind list and sort it by category.
- add an inventory screen as a tab on the bottom, like the help page and message log. make it really simple - just a simple list that you can page through with a cursor.
- add a "commands" screen as a tab on the bottom, that lets you perform macro actions like walking for 2,000 steps or walking to a specific coordinate.
- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI
- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI
- fix `make dev`
 - eventually, I'd like to publish this game on steam for $3. don't add/edit any code, I just want advice. What should I focus on before that?
- give me suggestions for different biomes!


- add random but seeded structure generation - old ruins, ore patches, machines, crashed ships, behemoth corpses, etc. Have the structure definition files be stored in ./lithicrivers/data/structures/ and come up with a reasonable format for them that lets you use ASCII art to define structures. for example:

/lithicrivers/data/structures/small_ship.lrstructure/
/lithicrivers/data/structures/small_ship.lrstructure/shape.txt

.XXX
X  A
XXX.

/lithicrivers/data/structures/small_ship.lrstructure/data.json
{
  "blocks": {
    ".": "empty",
    "X": "iron_scrap",
    "A": "door",
  },
  "gen_biomes": "ALL",
  "gen_chance": 0.001
}