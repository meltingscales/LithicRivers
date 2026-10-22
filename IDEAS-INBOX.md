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

- ANSWERED (switch saves from JSON to TOON?): **no, compress the JSON instead.** Measured on a new world (seed 12345, 59 chunks):
  - | Format | Size |
    |---|---|
    | Pretty JSON (what `save_to_file` writes today) | 5.87 MB |
    | Compact JSON | 1.73 MB |
    | MessagePack (`save_game_msgpack`, unused) | 1.24 MB |
    | Compact JSON with TOON-style tile lists | about 1.25 MB (the tile lists alone shrink 28%) |
    | Compact JSON, gzip -9 | 19 KB (300x smaller than today's file) |
  - 99% of a save is chunk tiles stored as strings (`"Rock","Rock",...`); 241,664 tiles are only 10,274 runs of the same tile, so any compression wins big
  - TOON con: made to save LLM tokens, not disk; young, with no mature serde support in Rust; only about a quarter smaller here; harder to edit than JSON. Pro: easier to read than JSON for tables. That isn't worth a second format
  - Suggested change if you want it, `compress-saves`: write compact JSON through gzip (the `flate2` crate), load both gzipped and plain saves so old saves keep working, and optionally store tiles as runs or palette indexes later

- DONE: went through STEAM-MVP.md, TODO.md, MECHANICS.md, QUESTS.md and docs/dungeons for ideas with no spec or change yet; they're listed, ranked for the Steam MVP and each with a suggested change name, in `docs/SPEC-GAPS.md`. Top picks: player death (it only logs "You have died!" today), a day/night cycle, the arm-replacement quest, and body damage with real effects. STEAM-MVP.md links the list, and volume sliders are ticked as done

- FIXED: turning Look mode off snaps the view back to the player
- DONE: the Look panel shows what is on the layers above and below the cursor (a vertical conveyor's own direction shows as "Lifts to" / "Lowers to" instead)
- FIXED: the Look panel puts its text first and wraps it, names things plainly ("Vertical conveyor (up)", "Log x2 (lying here)"), shows one 12x8 art (the entity's, else the tile's) last, and in Look mode the help box below it shrinks to fit, so nothing important is cut off
- DONE: in Look mode, hovering the mouse over the world picks what to look at; only the keys move the view

- PROPOSED as `add-underground-belts`: underground conveyor pairs on one layer, up to 10 tiles apart, carrying items under whatever lies between so belt lines can cross. Pipes wait until the game has fluids
- PROPOSED as `add-blueprint-sharing`: `LITHICRIVERS-BLUEPRINT-<payload>` strings (base64 of compressed JSON with a format version) to export and import blueprints and books; blueprint books as a tree with move, rename and delete; an edit pane that removes every block of one kind from a blueprint
