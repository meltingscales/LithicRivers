# Cursor Rules for LithicRivers

Make sure to also read docs/AI_INSTRUCTIONS.md.

## Project Context
This is a Python-based game development project called "LithicRivers". The project appears to be a roguelike or simulation game with:
- Sprite-based graphics system
- Sound and music assets
- Entity management
- Tile-based world system
- Structure generation

Please avoid running `cargo clean`. It takes a long time to rebuild rust modules.

Keep small copyrights inside of the main game code files. Make sure they exist if they aren't declared.

Guiding principles:

- All randomly-generated choices, actions, damage, loot, worldgen, etc - should be fully deterministic and based on world seed and world tick. This means that this game should be fully TAS-able for any specific initial seed (and version of game code). Always use the seed and any permutation (when appropriate) of XYZ coordinate, biome, or other seeded randomness to generate anything "random".

- This is supposed to be a mix between Caves of Qud, Factorio, and Minecraft. 
  - Cataclysm Dark Days Ahead is another fantastic game.
  - Gnomoria was cool.

- This game renders using ASCII text.

- TODO.md contains a big, unedited list of my long-term goals with this game. By no means do we need to implement all of them. I will likely cut or edit parts of the TODO. I just want to focus on the section called "MVP for steam release (2026)".

- We're currently in the process of migrating from Python to Rust. Please see the migration plan for more information, called "migration-plan.md" and "TODO-RUST-MIGRATION.md".

## Migration Plan

# LithicRivers Architecture

> **Note:** This document focuses on the technical architecture. For release planning, see [STEAM-MVP.md](STEAM-MVP.md).

### High-level Goals
- **Performance**: Optimize for smooth gameplay on target hardware
- **Maintainability**: Clean, documented code with clear separation of concerns
- **Modularity**: Components should be loosely coupled where possible
- **Determinism**: Game state must be fully deterministic for replay/TAS support

### Core Architecture
- **ECS (Entity Component System)**: Using `hecs` for game object management
- **Deterministic Simulation**: Fixed timestep game loop
- **Chunked World**: 3D grid-based world with efficient loading/unloading
- **Serialization**: `serde` with MessagePack for save games

### Target architecture
- Workspace layout
  - `crates/core`: ECS, components, resources, systems, RNG helpers, tiles, view model, structure loading.
  - `crates/client`: ratatui TUI for terminal rendering/input; consumes read-only state from `core` and loads sprites from assets.

- ECS (hecs)
  - Entities: IDs only.
  - Components: `Position`, `Renderable` (later), `Body`, `Inventory`, etc.
  - Resources (singletons): `World` (chunks, tiles, fluids), `Rng`, `MessageLog`, `Config`, `Keymap`.
  - Systems: movement, mining, inventory/pickup, AI/NPC, fluids, logging.
  - Schedule: ordered systems per fixed-tick with custom scheduling; client rendering reads state after sim.

- Rendering (ratatui + sprite assets)
  - TUI client renders colored ASCII using sprite metadata in `crates/client/assets/sprites/**/<name>.lrsprite`.
  - `crates/client/src/sprite_loader.rs` loads `data.json` + `sprites.txt` per sprite to choose a glyph and color.
  - `crates/core/src/view.rs` builds a read-only window of entity glyphs around the player; client overlays tiles/fluids/entities and prefetches chunks.
  - Client includes Inventory and Body panels; Body panel has an ASCII schematic plus a parts list.

- Determinism
  - `rand_chacha::ChaCha20Rng` as canonical RNG.
  - Position-based seeding for worldgen/structures/dungeons.
  - Stable iteration via `indexmap` or explicit sorting when order matters.
  - Single-threaded sim by default; parallelize worldgen or chunk-partitioned work carefully.

- Serialization
  - Rust: Not implemented yet. Target is `serde` with `rmp-serde` (MessagePack) for production and `serde_json` for debug.
  - Python (legacy): uses `msgspec` with MessagePack containers (`SavedGameData`, `SaveMetadata`).

### Data-model mapping from Python
- Keep: chunked world grid; integer fluids with settled flags; seeded worldgen; entity glyphs; deterministic tick loop.
- Change: replace class hierarchies with ECS components/systems; event listeners become ordered systems/queues.

### Migration phases (incremental)
1. Skeleton & minimal loop — DONE
   - `crates/core` with `Game` and ECS (`hecs`), deterministic RNG; `crates/client` ratatui app with input handling.
2. Core data shapes — DONE
   - Components: `Position`, `Player`, `Glyph`, `BlocksMovement`, `Sheep`; model `Body` present.
   - Resources: `Resources` with `World`, RNG, intents, fluids, tick; view builder exists.
   - Tiles: `TileKind` enum implemented; palette keys wired to sprites.
3. Save/Load — TODO
   - Implement `serde` data structs and versioning in `core`; add client commands to save/load.
4. Interaction & mining — PARTIAL
   - Input → `player_move_intent` → `move_player_system()` implemented. Basic mining converts trees and drops Wood; `pickup_system()` moves `DroppedItem` into `Inventory`.
5. Fluids — DONE (initial)
   - Integer fluids with `settled` optimization, thresholds/viscosity, directional spread, per-tick processing.
6. NPC/AI & messaging — PARTIAL
   - Simple `stumbling_sheep_system()` provides deterministic wandering. Message log exists and is rendered in the client.
7. Worldgen — PARTIAL
   - Infinite chunked world; deterministic per-chunk sprinkle; loads example structures in chunk (0,0) from `crates/client/assets/structures/*`.
8. Performance passes — TODO
   - Add profiling, consider chunk-frontier fluid processing, data layout tweaks.

### Next steps
- Build and run the client: `just build` then `just client` (press `q` to quit).
- Implement Rust save/load in `crates/core` using `serde` + `rmp-serde`; add keybinds in client for save/load.
- Expand mining and items: more tile interactions, item kinds/stacking rules, and item sprites.
- Flesh out worldgen: position-based seeding for structures/dungeons independent of chunk size; move demo placements into seeded generators.
- Polish message log UI; surface blocked-move feedback from `Resources.last_blocked_tile`.

### Legacy Python feature audit (for parity)
This section summarizes notable features implemented in the legacy Python codebase (`python-old/`) to ensure they are represented in the Rust migration. Status legend: DONE, PARTIAL, TODO.

- Fluids system — integer amounts with settling optimization
  - Python: `lithicrivers/game/fluids.py`
  - Rust: implemented initial integer fluids with `settled` flag and spread rules (see `crates/core`).

- UI system and popups (asciimatics)
  - Python: `lithicrivers/ui.py` includes dialogs, help, inventory, world map, message log, and popup manager.
  - Special case: numlock detection warning via `_detect_numlock_issue()`.
  - Rust: ratatui client present with Inventory/Body panels and message log; popups and numlock warning not yet implemented.

- Config and keybindings with platform-specific keychords
  - Python: `lithicrivers/config_manager.py`, `lithicrivers/keymap.py`, and script `scripts/generate_platform_keychords.py`.
  - Rust: input handling exists; configurable keybinds and platform keychords need design/port.

- Steam API integration (optional)
  - Python: `lithicrivers/model/steam_api.py` with availability checks and user info.
  - Rust: not implemented.

- Procedural dungeon generation (multiple types)
  - Python: `lithicrivers/procedural_dungeon_generator.py` supports cave systems, mining shafts, crypts, labs; deterministic via seeded RNG.
  - Rust: worldgen partial; move toward seeded, position-based dungeon/structure generation.

- Structure generation and worldgen determinism
  - Python: `lithicrivers/structure_generator.py`, `lithicrivers/worldgen.py`; position-based seeding to ensure chunk-size independence.
  - Rust: partial sprinkle/structure loading with target of fixed-grid + position-based seeds.

- Sprite loading, validation, and caching
  - Python: `lithicrivers/sprite_loader.py` validates dimensions, scales, caches sprites.
  - Rust: `crates/client/src/sprite_loader.rs` loads `data.json` + `sprites.txt`; ensure validation and caching parity.

- Entities, items, dropped items, and basic inventory
  - Python: `lithicrivers/game/entities.py` defines `Entity`, `DroppedItem`, `Fluid`, simple NPCs; inventory interactions and rendering.
  - Rust: basic items (e.g., Wood), `DroppedItem`, pickup and inventory systems are present; expand item kinds/stacking rules and sprites.

- NPCs and simple AI
  - Python: `lithicrivers/game/npcs.py` with behaviors; events in `game/events.py`.
  - Rust: simple wandering system; expand behaviors and eventing/message integration.

- Body system and UI
  - Python: `lithicrivers/model/body.py`, demos in `examples/body_page_demo.py` and tests.
  - Rust: `Body` model present; client has Body panel; continue parity for parts list and schematic.

- Message log and feedback
  - Python: message logging integrated across systems.
  - Rust: message log exists and is rendered; add more feedback events (e.g., blocked move, mining results).

- Save/Load parity and formats
  - Python: `lithicrivers/game/game_save_manager.py` with MessagePack via `msgspec` (legacy).
  - Rust: target `serde` + `rmp-serde` for production and `serde_json` for debug with versioned schemas.

- Test infrastructure and shared fixtures (legacy)
  - Python: optimized fixtures (`test/test_fixtures.py`), body/dungeon/sprite tests, and performance-tuned worldgen tests.
  - Rust: create analogous tests incrementally as features land.

## Steam MVP Release Plan (Target: December 2024)

### Small temp list

- fix windows exe consuming keystrokes way too fast (should we switch shells on windows?? does this only happen because im in a vm??)

- fix steam_cmd for steam release... see https://stackoverflow.com/questions/79740593/steamworks-vague-steamcmd-error-when-publishing

- reduce the amount of fluid in the Lithic Rivers biome, make a "spawn_lava"/"spawn_water"/etc method that just creates 1 standard block of lava with a slightly-lower-than-average volume so it doesnt spread like crazy as it currently does

- also clamp process_fluids_clamped so that it only does up to 50,000 fluid updates per tick, and quits with a warning if it hits that limit...

- add some more empty space to the Lithic Rivers biome, carve out meandering "cave room" and "empty river" paths using noise/erosion plus thick vectors

- arboreal biome
- frozen wasteland biome
- lush forest biome with springs and pools, and ruins
- nuked city biome
- abandoned mine biome

- forced structure spawns just like the python version
  - crashed ship
  - abandoned factory
  - abandoned research station

### Core Gameplay Loop (Must Have)
- [ ] Basic world generation with multiple biomes
  - [ ] At least 3 distinct biome bands with tile/feature differences
    - [ ] structure generation that's unique across biomes
    - [ ] underground "Lithic Rivers" biome that's molten lava and rare ore, and dangerous mining bots
    - [x] Deterministic by seed across Z-slices
- [ ] Player movement and basic interactions
  - [ ] Numpad movement plus '<'/'>' vertical movement
  - [ ] Mining action with SFX and message log feedback
- [ ] Simple crafting system
  - [ ] 3 starter recipes (stick, plank, torch) with deterministic outputs
- [ ] Day/night cycle
  - [ ] Full cycle length ~10–20 minutes with visual cue
- [ ] Basic survival mechanics (damage, repair, body parts)
  - [ ] Body panel shows part states; player can incur and repair at least 1 damage type
- [ ] NPC interactions and conversations just like the python version

### Technical Requirements
- [x] Stable save/load system
- [x] 3D world slices (Z-level viewing and vertical movement)
  - [x] View snaps to player Z after movement (toggle later if needed)
  - [x] Worldgen Perlin noise includes Z; slices deterministic by seed
- [ ] Windows build pipeline
  - [ ] CI job builds Windows artifacts
  - [ ] Smoke test launch on artifact
  - [ ] Upload artifacts to Releases (draft)
- [ ] Basic settings/controls menu
  - [ ] Rebind keys (movement, vertical, snap toggle)
  - [ ] Volume sliders (music/sfx)
- [ ] Performance optimizations for target hardware
  - [ ] Profiling budget: ~60 FPS at 80x24; ≤16 ms tick under normal load
  - [ ] Identify top 2 hotspots and add targeted optimizations

### Steam Integration
- [ ] Steamworks SDK integration
- [ ] Achievements system
- [ ] Cloud saves
- [ ] Basic Steam overlay support

### Polish & UX
- [ ] Tutorial/intro sequence
- [ ] Basic sound effects, not just music
- [ ] Main menu with new game/load game
- [ ] Basic UI feedback for player actions

### Post-MVP (After Release)
- [ ] Multiplayer support
- [ ] Advanced crafting system
- [ ] More biomes and world features
- [ ] Advanced AI behaviors
- [ ] Expanded building mechanics

### Timeline
- [ ] September: Core gameplay implementation
- [ ] October: Steam integration and performance
- [ ] November: Polish and bug fixing
- [ ] Early December: Beta testing
- [ ] Mid-December: Release

## Team
- kaya: 15% cut, music
- beezzaroll: 10% cut, art
- meltingscales: 75% cut, programming

## Development Priorities

### game
- add procedurally generated catgirls
- allow player to place blocks
- pushboxes
- audio system
  - music
- crafting
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

### Pre-steam-release TODO:
- steam SDK integration
- achievements system
- steam App ID
- Steam build pipeline
- steam store page

### UX and polish
- save system with auto backups
- settings and options menu
- sound!
- tutorial and onboarding
- error handling
- performance...lmao its a TUI

### Content
- core gameplay loop
- progression system
- content volume: At least a few hours of content. we should strive for proc-gen stuff because theres more replayability
- modding system??
- replayability

### Steam-specific features
- achievements
- trading cards
- cloud saves
- controller support
- steam workshop

### Marketing
- store page stuff
- community hub
- presskit

### Legal
- ToS
- privacy policy
- age rating
- tax info

### QA
- bug testing
- perf testing
- user testing
- steam deck compat

### Priority Order for Your Project:
- Complete the save system (already in your TODO)
- Add Steam SDK integration
- Polish the core gameplay loop
- Add achievements system
- Create compelling store page content
- Implement cloud saves
- Add controller support
- Set up Steam Workshop for mods

### Eventually todo
- nixos term resize causes crash, but not ubuntu. TODO test windows.
- Fix ubuntu numlock warning anomaly
- generate ubuntu keychords manually as they were lazily copied from nixos
- add the ability to play music in the game
- should I use cython to speed up worldgen? Does that cause some lag when objects get passed between python and c? Does threading cause issues or add lag?
- print logo briefly when game boots (config/logo.txt)
- print boot log briefly when game boots (config/boot_message.dat), making sure to add corruption. Make sure to print it slowly. Create a custom UI element to do this. Also, add a "SKIP_INTRO" env var to skip this for development purposes.
- if numlock is not toggled, the game crashes if you try to move.
- how should I cleanly handle SHIFT and CTRL and ALT keys? How do they appear as KeyboardEvents? A tuple of ints corresponding to key codes? If so, I need to change how my keymapping code works and the json structure. Probably add a second file (for easy reference, that doesnt get parsed) called "keycode_reference.json" that just maps {20=>A,21=>B}, etc, so people can easily read it. Have it get automatically generated by `lithicrivers/util/generate_keycode_reference.py`. when adding significant features
- Document any new asset formats or file structures
- Keep TODO.md updated with current development priorities
