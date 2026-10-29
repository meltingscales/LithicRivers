# Steam MVP Release Plan (Target: December 2026)

## quick list

- Ideas from these docs that have no spec or change yet are listed in [SPEC-GAPS.md](SPEC-GAPS.md).

## NEXT TODO: Underground Bunker Tutorial & First Quest Design
**Priority**: HIGHEST - First quest and tutorial implementation

**Quest 1: Underground Bunker Escape (z=-20 to surface)**
- Robot protagonist starts in a dark, unpowered bunker after falling and sustaining severe mechanical damage
- Low-light/no-light scenario with fog of war mechanics:
  - Structures/blocks are remembered and rendered normally once seen
  - Entity last positions are remembered and rendered in grayscale
  - Creates atmospheric tension and teaches core navigation mechanics
- Tutorial integration for movement, inventory, damage assessment, and basic interaction
- Environmental storytelling about how/why robot fell and bunker's purpose
- Escape to surface serves as clear objective and completion milestone

**Quest 2: SapienCorp Facility - New Arm Acquisition (1 mile from bunker)**
- Much more manageable 1-mile travel distance from bunker exit
- Maintains the abandoned factory quest structure detailed below
- Robot seeks prosthetic arm replacement after bunker damage

**Why This Change**: Eliminates the 10-mile walking simulator problem while creating immediate engagement through atmospheric underground escape sequence. The vertical progression teaches z-level mechanics naturally.

## Quest Location Design - Abandoned SapienCorp Factory
**Priority**: HIGH - First quest location implementation

first quest and associated dungeons

- design a multi-level dungeon that is the first quest as a structure (see `docs/STRUCTURES.md`)
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
1. (DONE) **Structure**: the SapienCorp factory's levels (`sapiencorp-factory.lrstructure`, hand-written layers since the VoxelBuilder import was retired)
2. (DONE) **Authoring tools**: structure plans, the lint test and the preview (`docs/STRUCTURES.md`)
3. (DONE) **Quest Integration**: the broken android sends the robot to the factory for a new arm (`arm-replacement-quest` spec)
4. (DONE) **Stairs Implementation**: Add Stairs block for vertical traversal between levels
5. **Environmental Storytelling**: Place lore elements and visual clues about SapienCorp's downfall

**Why This Matters**: Creates the first major quest destination and establishes the asset pipeline for structured locations. The abandoned factory serves as the game's first major narrative and gameplay milestone.

## TODO: Active Quest System Implementation
**Priority**: HIGHEST - Critical for tutorial quest and Steam MVP first impression

**Current Issues**:
- Quest can be started multiple times (dialogue doesn't track quest state)
- No way to deliver items to quest giver (fetch quest cannot be completed)
- Quest markers persist forever (no completion detection)
- No active quest tracking UI for players

**Implementation Plan**:
1. **Quest State Enum**: `NotStarted`, `Active`, `Completed`, `Failed`
2. **Active Quest System**: Track current quest objectives, required items, completion state
3. **Quests Tab**: New UI tab next to Global Map showing active/completed quests
4. **Quest Completion Detection**: Check player inventory for required items when talking to NPCs
5. **Item Delivery System**: Allow giving items to NPCs through dialogue interactions
6. **Quest State Integration**: Update dialogue system to prevent duplicate quest starts
7. **Marker Management**: Clear quest markers when quests are completed

**Quest Data Structure**:
- Quest ID (tied to QuestType enum)
- Current state (NotStarted/Active/Completed/Failed)
- Objective description
- Required items (for fetch quests)
- Reward items
- Associated NPC entity
- Quest marker position

**Why This Matters**: The tutorial quest is the first thing new players experience. It must work flawlessly - players should be able to accept it, track progress, deliver items, and see completion. Without this, the Steam MVP fails at the first hurdle.

## (DONE) Survival Mechanics Implementation

The survival loop works: explore, fight, take damage, repair. Combat damages specific body parts, the Body tab shows each part's condition, the repair modal turns crafted items into repairs, and every part's damage has a clear effect (`body-damage-effects` spec): legs slow walking, arms slow mining and soften melee, the head dims the light, and a damaged power source slows combat energy recovery. Damage types (physical, electrical) were left out.

## ugly combat UI

- the combat UI is kind of ugly
  - way too much spacing, remove margins on buttons
  - remove emojis
  - remove "z" char
  - rework it with Kaya

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
- [x] Day/night cycle
  - [x] **15-minute day, dark surface nights with fog of war, a clock in the MODES panel**
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
  - [x] Volume sliders (music/sfx)
    - [x] **Music and sound volume on the title screen's Options**
- [ ] Performance optimizations for target hardware
  - [ ] Profiling budget: ~60 FPS at 80x24; ≤16 ms tick under normal load
  - [ ] Identify top 2 hotspots and add targeted optimizations
  - **Baseline (2026-10-28, `just util-bench`, release build, AMD Ryzen AI Max+ 395):** every scenario within budget. Worst tick / worst frame in ms: deep-underground 10.32 / 0.64, factory-busy 1.07 / 0.30, night-pass-time 2.12 / 0.52, surface-walk 7.46 / 0.52. Medians are 0.1 to 1 ms; the worst ticks are one-off spikes when the player reaches new chunks, and those are the ones a slower CPU (the Steam Deck) would push over 16 ms.

## Steam Integration
- [ ] Steamworks SDK integration
- [ ] Achievements system
- [ ] Cloud saves
- [ ] Basic Steam overlay support

## Polish & UX
- [x] intro sequence
  - [x] **Boot message and intro screens (v0.7.6.2)**
- [x] tutorial sequence that can be accessed anytime
  - [x] **Interactive Tutorial System with modal overlays, popup GUI and keybind detection**
  - [x] **TutorialSystem class with toggleable panel (F1) and Menu integration**
- [ ] Basic sound effects, not just music
- [x] Main menu with new game/load game
  - [x] **Title screen, world list (play, rename, delete) and Create New World with per-world settings**
- [x] Basic UI feedback for player actions
  - [x] **Save/load keybinds displayed in menu**
  - [x] **Cross-platform NumLock detection and warnings**
  - [x] **Colored message system for better user feedback**
  - [x] **Context help panel on the World page (toggle with h)**

## Launch checklist
Non-code work from the old TODO.md.

- **Steam:** App ID, store page, community hub, trading cards, press kit.
- **Legal:** terms of service, privacy policy, age rating, tax info.
- **QA:** bug testing, performance testing, user testing, Steam Deck compatibility.
- **Price:** about $3.

## Launch outreach

**Steam key giveaway list:**
- skomor123
- noahnogueras@gmail.com
- oglingling
- college friends
- miguel from colab
- Hassan a
- cam 1
- cam 2
- cameron (false)
- https://steamcommunity.com/id/HoneyBottomFairy/ - lipe rineheart

**Streamers:**
- caseoh

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

2. **~~Day/night cycle~~** ✅ **COMPLETED**
   - ~~Full cycle length ~10–20 minutes with visual cue~~
   - ✅ **15-minute day; surface nights use fog of war; dogs are bolder at night**

3. **~~NPC interactions and conversations~~** ✅ **COMPLETED**
   - ~~Port from Python version for quest/story content~~
   - ✅ **Full dialogue system with portraits and emotional states implemented**

### MEDIUM PRIORITY (Polish & Performance)
4. **Performance optimizations**
   - Profiling budget: ~60 FPS at 80x24; ≤16 ms tick under normal load
   - Identify and fix top 2 hotspots

5. **~~Volume sliders (music/sfx)~~** ✅ **COMPLETED**
   - ✅ **Music and sound volume on the title screen's Options**

6. **Tutorial sequence**
   - ✅ **Interactive Tutorial System with modal overlays, popup GUI and keybind detection**
   - ✅ **TutorialSystem class structure with folder organization in crates/client/src/tutorialsystem/**
   - ✅ **Toggleable panel (F1 key) with Menu integration and auto-hide functionality**
   - Accessible anytime for new players, defaults to enabled
   - Critical for Steam release onboarding

### LOWER PRIORITY (Nice to Have)
7. **Main menu with new game/load game**
8. **More biome variety and structure generation**
9. **Basic sound effects** (beyond just music)
