# Steam MVP Release Plan (Target: December 2024)

## Small temp list

- `world_seed % 1` will randomly spawn a friendly Greyhound dog named either Kopi or FortyNiner that follows you around and attacks anything that attacks you. If you finish the game with the dog alive and in  your party, you get a free copy of the game to give to a friend, as well as some poetry about my early childhood and my thoughts about my favorite music artists.

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


## Core Gameplay Loop (Must Have)
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

## Technical Requirements
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

## Steam Integration
- [ ] Steamworks SDK integration
- [ ] Achievements system
- [ ] Cloud saves
- [ ] Basic Steam overlay support

## Polish & UX
- [ ] Tutorial/intro sequence
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
