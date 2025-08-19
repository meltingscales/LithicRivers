# Steam MVP Release Plan (Target: December 2024)

## Small temp list

- fix windows exe consuming keystrokes way too fast (should we switch shells on windows?? does this only happen because im in a vm??)

## Core Gameplay Loop (Must Have)
- [ ] Basic world generation with multiple biomes
  - [ ] At least 3 distinct biome bands with tile/feature differences
    - [ ] structure generation that's unique across biomes
    - [ ] underground "Lithic Rivers" biome that's molten lava and rare ore, and dangerous mining bots
    - [ ] Deterministic by seed across Z-slices
- [ ] Player movement and basic interactions
  - [ ] Numpad movement plus '<'/'>' vertical movement
  - [ ] Mining action with SFX and message log feedback
- [ ] Simple crafting system
  - [ ] 3 starter recipes (stick, plank, torch) with deterministic outputs
- [ ] Day/night cycle
  - [ ] Full cycle length ~10–20 minutes with visual cue
- [ ] Basic survival mechanics (damage, repair, body parts)
  - [ ] Body panel shows part states; player can incur and repair at least 1 damage type

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
