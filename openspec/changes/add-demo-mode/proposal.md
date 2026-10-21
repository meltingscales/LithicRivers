# Proposal

## Why

The title screen can't show what the game is like until you start a world. SNES games like Super Metroid ran an attract mode: left alone (or asked), they played short cuts of real gameplay, such as exploring, fighting and a boss, under a "DEMO" banner. LithicRivers has a deterministic headless player and scripted scenarios already, so it can do the same: a Demo button that plays short, real scenes of the game for players, streams and the Steam page.

## What Changes

- **Watch Demo** on the title screen, between Play Game and Options.
- **Scenarios**, each a real game world driven by a script, played back in real time and cut from one to the next:
  - **Exploring:** the robot walks out of the bunker area and across the surface, with fog of war lifting.
  - **Talking:** a conversation with an NPC, ending in an accepted quest.
  - **Combat:** a fight with feral dogs.
  - **Building:** placing blocks, belts and a chest.
  - **Factories:** the Factory showcase running, with belts moving, the tree farm planting and chopping, and the furnace lit.
- **DEMO banner:** a red bar across the top with "DEMO   DEMO   DEMO   ..." scrolling horizontally, styled after CICADAMATA's bold, minimal look (see `docs/ART-DIRECTION.md`). The scenario's name shows on the right.
- **Escape** ends the demo and returns to the title screen. Other keys do nothing, so the demo can't be played by accident.
- **Nothing is saved:** demo worlds never touch the saves folder or the config.

## Capabilities

### New Capabilities
- `demo-mode`: The Watch Demo choice, the scenarios and their playback, the banner, and leaving the demo.

### Modified Capabilities
<!-- None. The Watch Demo choice is specified in `demo-mode`; `title-menu`'s requirements still hold. -->

## Impact

- **Client:** a demo player that runs a scenario script step by step on `app.clock`, a `SplashState::Demo` screen with the banner, the title choice, and scenario scripts under `crates/client/assets/demos/`.
- **Headless:** scenario scripts use the existing script step format, so each one also runs as a snapshot test and stays working as the game changes.
- **No core changes:** scenarios use cheats, debug world types and keybinds that already exist.
