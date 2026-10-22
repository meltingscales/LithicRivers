<!-- AI don't remove these beginning lines please -->

# Ideas inbox

## Start

Put new feats here :) 

Ask for "Add an openspec feat that adds XYZ" or "Bugfix: Something not rendering"

## TODO ITEMS

<!-- AI feel free to edit below -->

<!-- MY THOUGHTS -->

.

<!-- ACTIONED ON ALREADY -->

- DONE as `add-mouse-menus`: click any row in any menu, list, form or window (title screen, world list, Choose Interaction, dialogue, filter and block pickers, combat moves, cheat suggestions, crafting, Item Dictionary); right-click moves a whole stack in item windows; clicking outside a window closes it; hover highlights rows. Widgets record their click areas while drawing, so clicks always match what's on screen

- DONE (feature/machine-window-filter): machine windows list only the items the machine takes, titled like "Showing fuel: 2 of 7". Generators show fuel, planters acorns, harvesters nothing; the furnace list follows the selected slot (fuel, smeltables, nothing for the output). Chests and corpses still list everything

- PROPOSED as `add-mouse-building` (answers "how should we handle mouse input?"):
  - Factorio/Minecraft-style buttons: left click places the hotbar block, else interacts, else looks; right click breaks; middle click picks the block (selects its item and copies its facing); scroll cycles hotbar slots; Shift+scroll rotates.
  - Same reach as the keyboard, a hover highlight with a Place-mode ghost, clickable tabs and scrollable lists.
  - Mouse bindings live in the config, and headless scripts get `click` / `scroll` / `move` steps.

- PROPOSED as `add-blueprints`: copy an area into a blueprint, blinking paste preview with rotation, warning before replacing player-built blocks (they go to your inventory), paid from your inventory, blueprint library saved with the world

- PROPOSED as `add-nanobot-modules`: Mk1-Mk3 nanobot modules in torso slots build blueprints over time (more and better modules build faster and farther), drawing on the robot's power source; `nanobots_free` cheat for free, instant building

- PROPOSED as `add-teleporters`: Waystones-style teleporters with battery banks charged from the grid, a named destination list, jump cost growing with distance, and a "needs more charge" message when short

- PROPOSED as `add-vertical-conveyors`: up/down conveyors between layers (stackable, toggle with Rotate), same recipe as wooden conveyors plus one-for-one conversion recipes both ways

- FIXED (fix/machine-power-on-logistics): auto-planters and auto-harvesters drew almost nothing (a farm returned ~20x its fuel at ~2 kW average). Now the planter draws 60 kW for 4 s per acorn and the harvester 180 kW for 6 s per chop; together they need more than one burner generator, and a tree returns ~4x its cost

- PROPOSED as `add-demo-mode`: Watch Demo on the title screen; scripted explore / combat / build / factory scenarios cut together in a loop under a scrolling red CICADAMATA-style "DEMO" banner; Escape returns to the title

- DONE (feature/chest-take-all): "Take all" keybind (`ui.TAKE_ALL`, `a`) in chest, corpse, generator and machine windows

- PROPOSED as `add-factory-stats`: power graph per network in the pole info box (last 60 s / 10 min), item produced/consumed counts by source, and a Production tab

- PROPOSED as `add-item-dictionary`: Item Dictionary tab next to Crafting (sprites, descriptions, recipes, uses, full-text search), and cheat console suggestions as a scrolling list so `giveitem` names aren't cut off

- PROPOSED as `add-sprite-art`: real 12x8 art for all 73 sprites (every one is still a placeholder), fixes for small sprites that look alike, a test that fails on placeholder art, and a gallery snapshot

- PROPOSED as `add-belt-logistics`: extractor (chest/machine to belt), filter extractor, belt router (splits in turn, optional filter), and walkable belts that carry the player

- PROPOSED: unique LithicRivers tech tree. Roadmap in `docs/TECH-TREE.md`; inspirations added to `docs/INSPIRATION.md` and AGENTS.md
  - `add-ore-spouts`: molten ore spouts (underground, and on the surface at half the chance), fluids, pumps, pipe networks into machines, casting basins, crucibles, iron ingots
  - `add-ruined-reactors`: ruined reactor multiblocks, nuclear waste, uranium, thermoelectric generators (low, endless power)
  - advanced nuclear tier: outlined in `docs/TECH-TREE.md`, not proposed yet

- PROPOSED as `add-world-snapshots`: automatic snapshots, branching timeline with rollback, action log per snapshot, keep last 10 (+ current)

- FIXED (fix/scratch-items-2): "Choose Interaction" now shows each choice's direction (repro: `scripts/agent/bugs/choose-interaction-directions.txt`)

- FIXED (fix/scratch-items-2): `giveitem chest` with no quantity gives 1

- PROPOSED as `add-crafting-cheats`: `givecraftingcomponents <item> [amount] [raw]`

- PROPOSED as `add-debug-worlds`: World type in Create World (dev mode): Void, Ore test, Dog arena (100 dogs), Factory showcase (every factory block must have an example)
