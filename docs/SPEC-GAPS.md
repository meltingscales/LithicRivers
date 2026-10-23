# Spec gaps

AI Note: When an item is proposed, delete its row from SPEC-GAPS.md after it's migrated to openspec... The change takes over from there, so the file only ever lists ideas without a change.

Feature ideas from the docs that no OpenSpec spec (`openspec/specs/`) or open change (`openspec/changes/`) covers yet, as of 2026-10-22. Sources: `docs/STEAM-MVP.md`, `docs/TODO.md` (mostly its "MVP for steam release" section), `docs/MECHANICS.md`, `docs/QUESTS.md` and `docs/dungeons/`.

To act on one, ask for it by its suggested change name ("propose add-day-night-cycle"). Ideas that are already done or already have a change are listed at the end, so they aren't proposed twice.

## For the Steam MVP

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **Steamworks:** achievements, cloud saves and the overlay. | STEAM-MVP.md, TODO.md | Missing | `add-steamworks` |
| **Performance budget:** about 60 FPS at 80x24 and at most 16 ms per tick; profile and fix the top two hotspots, with a benchmark that guards them. | STEAM-MVP.md | Missing | `add-performance-budget` |
| **Combat panel cleanup:** too much spacing, emojis and a stray "z" character. | STEAM-MVP.md ("ugly combat UI") | Missing | `polish-combat-panel` |
| **Sound effects** beyond music, for actions other than mining. | STEAM-MVP.md | Partly | `add-sound-effects` |
| **Controller support and Steam Deck** checks. | TODO.md | Missing | `add-controller-support` |

## Worldgen and places

| Idea | Source | Today | Suggested change |
|---|---|---|---|
| **More surface biomes:** arboreal, frozen wasteland, lush forest with springs, pools and ruins, nuked city, abandoned mine. `overhaul-worldgen-depth` leaves the surface as it is on purpose. | STEAM-MVP.md (small temp list), TODO.md | Missing | `add-surface-biomes` |
| **Landmark structures** placed by worldgen, like the Python version's: a crashed ship and an abandoned research station (the abandoned factory exists). | STEAM-MVP.md | Partly | `add-landmark-structures` |
| **Procedural dungeons v2**, using the TinyKeep layout algorithm in `docs/dungeons/tinybuild-dungeons.md`. | TODO.md, docs/dungeons | Missing | `add-procedural-dungeons` |
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
| **Equipment and mobility:** jetpacks (fuel and electric), grappling hooks, fuel cells, electric and gas tools. | TODO.md | Missing | `add-mobility-gear` |
| **Push boxes** that need certain body parts to move. | TODO.md ("in progress") | Missing | `add-push-boxes` |
| **Programmable robots:** sandboxed scripting with sample programs found on dead robots (TreeCutter, StripMiner, Sentry). A large change; the docs mention Python, Lua or block logic. | TODO.md | Missing | `add-robot-scripting` |
| **Inventory comfort:** junk rules, auto-dump, and "are you sure" for destructive actions. | TODO.md | Missing | `polish-inventory` |

## Later, or flavor

- **A companion dog**, Kopi or FortyNiner, chosen by the world seed, that follows the player and fights alongside them. *(TODO.md)*
- **NPC tribes** that gather from the world and build machines on their own. *(TODO.md)*
- **Difficulty templates:** true roguelike, roguelite and custom. *(TODO.md)*
- **Autosave with crash durability** (a write-ahead log or frequent snapshots). Mostly covered by `add-world-snapshots`; check whether a crash mid-save is covered there. *(TODO.md)*
- **Relics and techmagic, multiple dimensions, space travel.** *(TODO.md)*

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
| Ore spouts, ruined reactors, nuclear tier | `add-ore-spouts`, `add-ruined-reactors`, `add-nuclear-slime` |
| Lithic Rivers depth, caves and deposits | `worldgen-depth` spec |
| Factorio-style hotbar, building that isn't annoying | `mouse-input`, `blueprints` and the factory specs |
| Player death, day/night cycle, the arm quest, body damage effects | `add-player-death`, `add-day-night-cycle`, `add-arm-replacement-quest`, `add-body-damage-effects` |
| Title screen, world list and load game | `title-menu`, `world-saves` |
