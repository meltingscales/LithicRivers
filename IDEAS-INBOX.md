<!-- AI don't remove these beginning lines please -->

# Ideas inbox

## Start

Put new feats here :) 

Ask for "Add an openspec feat that adds XYZ" or "Bugfix: Something not rendering" or "Feat: Add sheep grass eating behavior"

## TODO ITEMS

<!-- AI feel free to edit below -->

<!-- MY THOUGHTS -->

.

<!-- ACTIONED ON ALREADY -->

- DONE: went through STEAM-MVP.md, TODO.md, MECHANICS.md, QUESTS.md and docs/dungeons for ideas with no spec or change yet; they're listed, ranked for the Steam MVP and each with a suggested change name, in `docs/SPEC-GAPS.md`. Top picks: player death (it only logs "You have died!" today), a day/night cycle, the arm-replacement quest, and body damage with real effects. STEAM-MVP.md links the list, and volume sliders are ticked as done

- FIXED: turning Look mode off snaps the view back to the player
- DONE: the Look panel shows what is on the layers above and below the cursor (a vertical conveyor's own direction shows as "Lifts to" / "Lowers to" instead)
- FIXED: the Look panel puts its text first and wraps it, names things plainly ("Vertical conveyor (up)", "Log x2 (lying here)"), shows one 12x8 art (the entity's, else the tile's) last, and in Look mode the help box below it shrinks to fit, so nothing important is cut off
- DONE: in Look mode, hovering the mouse over the world picks what to look at; only the keys move the view

- PROPOSED as `add-underground-belts`: underground conveyor pairs on one layer, up to 10 tiles apart, carrying items under whatever lies between so belt lines can cross. Pipes wait until the game has fluids
- PROPOSED as `add-blueprint-sharing`: `LITHICRIVERS-BLUEPRINT-<payload>` strings (base64 of compressed JSON with a format version) to export and import blueprints and books; blueprint books as a tree with move, rename and delete; an edit pane that removes every block of one kind from a blueprint
