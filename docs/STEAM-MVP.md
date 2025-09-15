# Steam MVP Release Plan (Target: December 2026)

## NEXT TODO: VoxelBuilder Quest Location Design - Abandoned SapienCorp Factory
**Priority**: HIGH - First quest location implementation

first quest and associated dungeons

- use VoxelBuilder (https://nimadez.github.io/voxel-builder/) to design a multi-level dungeon that is the first quest
- use the existing `utility_voxelbuilder_importer.rs` to convert VoxelBuilder JSON exports to .lrstructure format
- add a Stairs block that you must use to traverse up/down in the world
- add randomly generated loot that you can use to replace your Arm that's guaranteed to spawn in a specific chest near the assembly line

**SapienCorp Facility Design - 3-Level Structure**:
- **Ground Level (Entry)**: ~20x15 tiles
  - Light combat encounters (1-2 enemies)
  - Basic repair station tutorial area
  - Environmental storytelling about the facility's purpose
- **Lower Level (Assembly Line)**: ~25x20 tiles
  - Main combat area with 3-4 enemies guarding the assembly line
  - Guaranteed arm replacement loot chest
  - Damaged machinery requiring repair skill demonstration
- **Sub-Level (Storage/Labs)**: ~15x12 tiles
  - Optional exploration area with additional loot and lore
  - 1-2 tougher enemies for advanced players
- **Total**: ~60x47 tiles across 3 levels with stairs connecting each level

**Implementation Plan**:
1. **VoxelBuilder Design**: Create the 3-level SapienCorp factory structure in VoxelBuilder
2. (DONE) **Import Pipeline**: Use existing `utility_voxelbuilder_importer.rs` to convert to .lrstructure format
3. **Quest Integration**: Connect the location to first major quest narrative
4. (DONE) **Stairs Implementation**: Add Stairs block for vertical traversal between levels
5. **Environmental Storytelling**: Place lore elements and visual clues about SapienCorp's downfall

**Why This Matters**: Creates the first major quest destination and establishes the asset pipeline for structured locations. The abandoned factory serves as the game's first major narrative and gameplay milestone.

## TODO: Survival Mechanics Implementation
**Priority**: HIGH - Essential for core survival gameplay loop

**Implementation Plan**:
1. **Damage System**: Combat affects specific body parts with varying damage types (physical/mechanical)
2. **Body Panel Integration**: Visual indicators showing part condition in existing UI
3. **Repair Mechanics**: Use crafted items or rest to restore damaged parts
4. **Resource Management**: Create meaningful trade-offs between combat risk and repair costs
5. **Gameplay Impact**: At least one damage type with clear mechanical consequences

**Why This Matters**: Completes the survival loop - explore → combat → damage → repair → explore. Currently combat has no lasting consequences, making it feel disconnected from the crafting/resource systems.

## launching

- find out how x-terminal-emulator works on Ubuntu
  - why is it tiny if steam launches it with crap font?
  - why does it look great when using gnome terminal?

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
  - [x] Make building/placing blocks/mining fun and fast!
    - [x] ~~vim-style build/break toggle?~~ **DONE: 'v' key toggles break/place mode (v0.8.2.0)**
    - [x] **Block placement with inventory consumption implemented**
- [x] Simple crafting system
  - [x] 3 starter recipes (stick, plank, torch) with deterministic outputs
- [ ] Day/night cycle
  - [ ] Full cycle length ~10–20 minutes with visual cue
- [ ] Basic survival mechanics (damage, repair, body parts)
  - [ ] Body panel shows part states; player can incur and repair at least 1 damage type
- [x] NPC interactions and conversations just like the python version
  - [x] **Dialogue engine with clean architecture and unit tests**
  - [x] **15+ emotional dialogue states for rich NPC interactions**
  - [x] **Summon Night-style dual portraits (NPC + player mood-based)**
- [x] Combat system with tick-based moves and an option to use items and run away
  - [x] Running away costs 600 ticks
  - [x] Tackling an enemy costs 800 ticks
  - [x] Tackling an enemy stuns it for 400 ticks (50% chance)
  - [x] Tackling an enemy pushes it back 2 spaces
  - [x] **Dead enemies turn into lootable corpses (v0.8.1.0)**
  - [x] **Combat fully merged and stable (v0.8.1.1)**

## Technical Requirements
- [x] Stable save/load system
  - [x] **Viewport position now saved/restored on load**
  - [x] **FeralDogs (and all entities) now persist correctly across save/load**
  - [x] **Save/load restricted to menu tab only for safety**
- [x] 3D world slices (Z-level viewing and vertical movement)
  - [x] View snaps to player Z after movement (toggle later if needed)
  - [x] Worldgen Perlin noise includes Z; slices deterministic by seed
- [x] Windows build pipeline
  - [x] CI job builds Windows artifacts
  - [x] Smoke test launch on artifact
  - [x] Upload artifacts to Releases (draft)
- [x] Basic settings/controls menu
  - [x] Rebind keys (movement, vertical, snap toggle)
  - [x] **F1-F12 hotbar assignment system (v0.8.2.0)**
  - [x] **Mode toggling blocked during combat**
  - [x] **Colored message log system with red NumLock warnings**
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
  - [x] **Boot message and intro screens (v0.7.6.2)**
- [ ] tutorial sequence that can be accessed anytime
- [ ] Basic sound effects, not just music
- [ ] Main menu with new game/load game
- [x] Basic UI feedback for player actions
  - [x] **Save/load keybinds displayed in menu**
  - [x] **Cross-platform NumLock detection and warnings**
  - [x] **Colored message system for better user feedback**

## Post-MVP (After Release)
- [ ] Multiplayer support
- [ ] Advanced crafting system
- [ ] More biomes and world features
- [ ] Advanced AI behaviors
- [ ] Expanded building mechanics

## Next Priority Items (Suggested Order)

### HIGH PRIORITY (Core Gameplay Gaps)
1. **Basic survival mechanics (damage, repair, body parts)**
   - Body panel shows part states; player can incur and repair at least 1 damage type
   - This is essential for the core survival loop

2. **Day/night cycle**
   - Full cycle length ~10–20 minutes with visual cue
   - Affects gameplay rhythm and difficulty

3. **~~NPC interactions and conversations~~** ✅ **COMPLETED**
   - ~~Port from Python version for quest/story content~~
   - ✅ **Full dialogue system with portraits and emotional states implemented**

### MEDIUM PRIORITY (Polish & Performance)
4. **Performance optimizations**
   - Profiling budget: ~60 FPS at 80x24; ≤16 ms tick under normal load
   - Identify and fix top 2 hotspots

5. **Volume sliders (music/sfx)**
   - Complete the settings menu
   - Important for user customization

6. **Tutorial sequence**
   - Accessible anytime for new players
   - Critical for Steam release onboarding

### LOWER PRIORITY (Nice to Have)
7. **Main menu with new game/load game**
8. **More biome variety and structure generation**
9. **Basic sound effects** (beyond just music)

## Timeline
- [ ] September: Core gameplay implementation (**Body mechanics, Day/night, NPCs**)
- [ ] October: Steam integration and performance (**Optimization, Tutorial**)
- [ ] November: Polish and bug fixing (**UI/UX refinements**)
- [ ] Early December: Beta testing
- [ ] Mid-December: Release
