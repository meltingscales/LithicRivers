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

- make trees guaranteed to drop at least 1 acorn, up to 3, but dropping 1 90% of the time.
- add a dialog box system, as well as a starter NPC that you can talk to and choose a dialog option at the end of a short conversation.
- unify keybinds to 1 file, perhaps being managed as part of a class that loads a JSON file. Have the keybinds be stored in a JSON file that can be modified by the user, and gets packaged as part of the exe.
- like caves of qud, use the numpad for 8-way movement. WASD can be used for something else later.
- color-code different elements within the TUI
- ask "are there any bugs or issues you see with my existing viewport code? can we stress test it?" to AI
- ask "Are there any bugs or issues you see with my world data structure? how does minecraft do it?" to AI
- limit the ability to zoom in to 1,2,3 scales.
- for viewscale=2, there's dots on the bottom-right of the viewport. find a way to clamp the viewport to a specific dimension that's the same as viewscale=1.
- fix `make dev`
- segregate the makefile targets into sections visually in the `help` target
- figure out why `make test` takes so damn long - do we need to optimize worldgen speed? or just make the sizes of our worlds in tests smaller?