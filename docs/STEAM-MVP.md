# Steam MVP Release Plan (Target: December 2026)

## launching

- find out how x-terminal-emulator works on Ubuntu
  - why is it tiny if steam launches it with crap font?
  - why does it look great when using gnome terminal?

## first quest and associated dungeons

- use goxel (see `crates/client/assets/goxel/test.gox`) to design a multi-level dungeon that is the first quest
- have a build step that exports .gox files to our weird custom .txt format
  - store it in `src/bin/utility_goxel_exporter.rs` or something, next to our demos
  - for now, just hardcode input/output paths in `utility_goxel_exporter.rs`...
- add a Stairs block that you must use to traverse up/down in the world
- add randomly generated loot that you can use to replace your Arm that's guaranteed to spawn in a specific chest near the assembly line

## Small temp list

- structure gen seems like it gets "stretched" across infinite z?

- fix windows exe consuming keystrokes way too fast (should we switch shells on windows?? does this only happen because im in a vm??)

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

## Core Gameplay Loop (Must Have)
- [ ] Basic world generation with multiple biomes
  - [ ] At least 3 distinct biome bands with tile/feature differences
    - [ ] structure generation that's unique across biomes
    - [ ] underground "Lithic Rivers" biome that's molten lava and rare ore, and dangerous mining bots
    - [x] Deterministic by seed across Z-slices
- [x] Player movement and basic interactions
  - [x] Numpad movement plus '<'/'>' vertical movement
  - [x] Mining action with SFX and message log feedback
  - [ ] Make building/placing blocks/mining fun and fast!
    - [ ] vim-style build/break toggle?
- [x] Simple crafting system
  - [x] 3 starter recipes (stick, plank, torch) with deterministic outputs
- [ ] Day/night cycle
  - [ ] Full cycle length ~10–20 minutes with visual cue
- [ ] Basic survival mechanics (damage, repair, body parts)
  - [ ] Body panel shows part states; player can incur and repair at least 1 damage type
- [ ] NPC interactions and conversations just like the python version
- [ ] Combat system with tick-based moves and an option to use items and run away
  - [ ] Running away costs 600 ticks
  - [ ] Tackling an enemy costs 800 ticks
  - [ ] Tackling an enemy stuns it for 400 ticks (50% chance)
  - [ ] Tackling an enemy pushes it back 2 spaces

## Technical Requirements
- [x] Stable save/load system
- [x] 3D world slices (Z-level viewing and vertical movement)
  - [x] View snaps to player Z after movement (toggle later if needed)
  - [x] Worldgen Perlin noise includes Z; slices deterministic by seed
- [x] Windows build pipeline
  - [x] CI job builds Windows artifacts
  - [x] Smoke test launch on artifact
  - [x] Upload artifacts to Releases (draft)
- [x] Basic settings/controls menu
  - [x] Rebind keys (movement, vertical, snap toggle)
  - [ ] Volume sliders (music/sfx)
- [ ] Performance optimizations for target hardware
  - [ ] Profiling budget: ~60 FPS at 80x24; ≤16 ms tick under normal load
  - [ ] Identify top 2 hotspots and add targeted optimizations

## Steam Integration
- [ ] Steamworks SDK integration
- [ ] Achievements system
- [ ] Cloud saves
- [ ] Basic Steam overlay support

## Polish & UX
- [x] intro sequence
- [ ] tutorial sequence that can be accessed anytime
- [ ] Basic sound effects, not just music
- [ ] Main menu with new game/load game
- [ ] Basic UI feedback for player actions

## Post-MVP (After Release)
- [ ] Multiplayer support
- [ ] Advanced crafting system
- [ ] More biomes and world features
- [ ] Advanced AI behaviors
- [ ] Expanded building mechanics

## Timeline
- [ ] September: Core gameplay implementation
- [ ] October: Steam integration and performance
- [ ] November: Polish and bug fixing
- [ ] Early December: Beta testing
- [ ] Mid-December: Release
