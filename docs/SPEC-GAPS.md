# Spec gaps

AI Note: When an item is proposed, delete its row from SPEC-GAPS.md after it's migrated to openspec... The change takes over from there, so the file only ever lists ideas without a change.

Feature ideas from the docs that no OpenSpec spec (`openspec/specs/`) or open change (`openspec/changes/`) covers yet, as of 2026-10-22. Sources: `docs/STEAM-MVP.md`, the old `docs/TODO.md` (folded in here and removed on 2026-10-22), `docs/MECHANICS.md`, `docs/QUESTS.md` and `docs/dungeons/`.

To act on one, ask for it by its suggested change name ("propose add-day-night-cycle"). Ideas that are already done or already have a change are listed at the end, so they aren't proposed twice.

## For the Steam MVP

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **Steam achievements:** the real achievement list (`add-steamworks` only adds a placeholder for dying). | STEAM-MVP.md | Missing | `add-steam-achievements` |
| **Combat panel cleanup:** too much spacing, emojis and a stray "z" character. | STEAM-MVP.md ("ugly combat UI") | Missing | `polish-combat-panel` |
| **Sound effects** beyond music, for actions other than mining. | STEAM-MVP.md | Partly | `add-sound-effects` |
| **Controller support and Steam Deck** checks. | TODO.md | Missing | `add-controller-support` |
| **Mods:** a modding system and Steam Workshop support. | TODO.md | Missing | `add-modding` |

## Worldgen and places

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **More surface biomes:** arboreal, frozen wasteland, lush forest with springs, pools and ruins, nuked city, abandoned mine. `overhaul-worldgen-depth` leaves the surface as it is on purpose. | STEAM-MVP.md (small temp list), TODO.md | Missing | `add-surface-biomes` |
| **Landmark structures** placed by worldgen, like the Python version's: a crashed ship and an abandoned research station (the abandoned factory exists). | STEAM-MVP.md | Partly | `add-landmark-structures` |
| **Procedural dungeons v2**, using the TinyKeep layout algorithm in `docs/dungeons/tinybuild-dungeons.md` (originally from https://www.gamedeveloper.com/programming/procedural-dungeon-generation-algorithm). | TODO.md, docs/dungeons | Missing | `add-procedural-dungeons` |
| **Mining bots:** dangerous enemies of the Lithic Rivers depths. | STEAM-MVP.md (core loop) | Missing | `add-mining-bots` |
| **Structure generation "stretched" across z**, noted as a bug. `overhaul-worldgen-depth` (archived) reworked the bands but did not check it. | STEAM-MVP.md (small temp list) | Bug | `fix-structure-z-stretch` |

## Getting around

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **Waypoints and a world map with fast travel**, like Caves of Qud's, where map tiles average their chunk. The Global Map tab exists; waypoints and travel don't. Teleporters (`add-teleporters`) are the late-game version. | TODO.md | Partly | `add-waypoints` |
| **A commands tab for macro actions:** walk 2000 steps, walk to a coordinate. Also the base for the screen-reader mode below. | TODO.md | Missing | `add-macro-commands` |
| **Screen-reader mode** (`LITHICRIVERS_BLIND_MODE=1`): a command lookup and help system tested with popular screen readers. | TODO.md | Missing | `add-screen-reader-mode` |

## Bodies, gear and automation

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **Equipment and mobility:** jetpacks (fuel and electric), grappling hooks, fuel cells, electric and gas tools, antigravity (with comedic "fall into space" mishaps) and falling. | TODO.md | Missing | `add-mobility-gear` |
| **Push boxes** that need certain body parts to move, and block placement limited by body parts. | TODO.md ("in progress") | Missing | `add-push-boxes` |
| **Programmable robots:** sandboxed scripting with sample programs found on dead robots (TreeCutter, StripMiner, Sentry). A large change; the docs mention Python, Lua or block logic. | TODO.md | Missing | `add-robot-scripting` |
| **Inventory comfort:** junk rules, auto-dump, and "are you sure" for destructive actions. | TODO.md | Missing | `polish-inventory` |
| **Auto-harvest and dig modes** for the player: harvest what you walk past, and dig through rock instead of getting stuck when moving between layers. | TODO.md (MVP section) | Missing | `add-movement-modes` |
| **Explosions and fire.** | TODO.md | Missing | `add-explosions` |
| **Fights capped at 3 enemies**, Chrono Trigger style, so players don't get swarmed. | TODO.md | Missing | `cap-combat-enemies` |
| **Dogs drawn to food and organic matter.** | TODO.md (MVP section) | Missing | `add-creature-attraction` |
| **Crafting ratios:** a view of recipe ratios for spreadsheet-style factory planning, like Factorio's. The missing-materials tracker exists. | TODO.md | Missing | `add-crafting-ratios` |
| **Variable Z rendering:** show several layers at once for tall buildings. | TODO.md | Missing | `add-layered-view` |

## Later, or flavor

- **A companion dog**, a greyhound named Kopi or FortyNiner, chosen by the world seed, that follows the player and fights alongside them. Finishing the game with the dog alive gives a free copy for a friend, plus poetry about the user's childhood and favorite artists (lapfox trax, 4lung). *(TODO.md)*
- **NPC tribes** that gather from the world and build machines on their own. *(TODO.md)*
- **Difficulty templates:** true roguelike, roguelite and custom. *(TODO.md)*
- **Autosave with crash durability** (a write-ahead log or frequent snapshots). Mostly covered by `add-world-snapshots`; check whether a crash mid-save is covered there. *(TODO.md)*
- **Branching saves view:** `add-world-snapshots` covers checkpoints; the TODO also wanted a `git log --graph` style view and a first-time hint like "This feels very useful. You can access this memory any time in the XYZ menu." *(TODO.md)*
- **Minecraft-style saves:** chunks, dimensions, a custom binary format, and multi-threaded loading and saving. *(TODO.md)*
- **Cybernetics and augments** beyond body parts; `add-nanobot-modules` and `add-multi-arm-bodies` cover some of it. *(TODO.md)*
- **A `SKIP_INTRO` env var** to skip the boot intro during development. *(TODO.md)*
- **Relics and techmagic, multiple dimensions (space, the void), space travel.** *(TODO.md)*
- **Main questlines:** 10 to 20 static questlines with their NPCs, buildings, items and dungeons, and finishing `story.txt`. Puzzles and hazards as challenges along the way. *(TODO.md)*
- **A stats web portal:** a separate site for game stats, the map, entities and performance. *(TODO.md)*
- **Gaussian noise for resources** (John's idea): scattered ore deposits, cave entrances, rare artifacts and weather events like acid rain or radiation storms, alongside Perlin noise for terrain. *(TODO.md)*
- **GPU worldgen** that stays deterministic (fixed-point compute shaders). *(TODO.md)*
- **Flavor:** procedurally generated catgirls; a nod to hushy and lapfox trax (music, sprites, items or tools); an "armless legless" challenge start; portable nukes with no consequences thanks to backup bodies. *(TODO.md)*
- **Research:** the `gale93/sbixel` repo, and how Dwarf Fortress handles vertical combat. *(TODO.md)*

## Design notes

Principles from the old TODO.md, for whoever writes the specs:

- There is no EXP: the robot's stats come from its equipment and body.
- Everything is craftable, and everything breaks down to raw materials.
- Automation is heavily rewarded but never required to finish the game.
- Prototype systems (inventory pages, UI, fights) with paper cutouts before building them.
- Content goals for release: a core gameplay loop, a progression system, at least a few hours of content, and replayability, leaning on procedural generation.

## Already done or already covered

These came up in the docs but need no new change:

| Idea | Where it lives |
|---|---|
| Stairs and climbing between layers | `vertical-conveyors` spec (stairs, climbing, the build-stairs prompt) |
| Fog of war and light | Implemented (`docs/MECHANICS.md`) |
| Quest states and the Quests tab | Implemented (`QuestState`, `ui/panels/quests.rs`) |
| Body repair | Implemented (repair modal, `RepairRecipeHandler`) |
| Branching saves and checkpoints | `add-world-snapshots` |
| A 3D world explorer | `add-3d-viewer` |
| Ore spouts, ruined reactors, nuclear tier | `add-ore-spouts`, `add-ruined-reactors`, `nuclear-slime` spec |
| Lithic Rivers depth, caves and deposits | `worldgen-depth` spec |
| Factorio-style hotbar, building that isn't annoying | `mouse-input`, `blueprints` and the factory specs |
| Player death, day/night cycle, the arm quest, body damage effects | `player-death` and `death-options` specs, `day-night-cycle` spec, `arm-replacement-quest` spec, `body-damage-effects` spec |
| Title screen, world list and load game | `title-menu`, `world-saves` |
| Fluids | `add-ore-spouts` (fluids spec) |
| Missing-materials tracker | `crafting` spec |
| Z travel only by stairs | `vertical-conveyors` spec |
| Self-limiting plant growth, crops | `tree-growth` spec, `add-crops` |
| Block logic and Lua scripting | `add-sapiennet`, `add-sapiennet-scripting` |
| Fuel cells | `add-power-cells` |
| Specialty bodies with more limb slots | `add-multi-arm-bodies` |
| Combat (tackle, escape delay, same-Z fights, corpses) | Implemented |
| Boot logo and boot log, seeded worldgen across Z | Implemented |
