<!-- AI don't remove these beginning lines please -->

# TEMP/SCRATCH

## Start

Put new feats here :) 

Ask for "Add an openspec feat that adds XYZ" or "Bugfix: Something not rendering"

## TODO ITEMS

<!-- AI feel free to edit below -->

<!-- MY THOUGHTS -->

.


<!-- ACTIONED ON ALREADY -->

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
