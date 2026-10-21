# Proposal

## Why

Blueprints (`add-blueprints`) start out instant and free of tools. That's fine early on, but building whole factories at once should be something the robot earns. The robot is made of parts (head, torso, power source, arms, legs); nanobot modules installed in its body are a natural, unique way to unlock and speed up remote building, with tiers through the mid and late game.

## What Changes

- **Nanobot modules:** a new kind of body upgrade, installed in the torso in the Body tab, with a few module slots.
  - **Tiers:** Mk1 (mid game), Mk2 and Mk3 (end game), each with its own recipe from later-tier materials.
- **Building with nanobots:** a blueprint paste needs at least one nanobot module. The paste is no longer instant: blocks are built one at a time, at a speed set by the total module power (more modules and higher tiers build faster), and appear in order with a short shimmer.
  - Building draws on the robot's power source, so it can't run forever.
- **Out of range:** nanobots build only within a radius of the player that grows with tier. Blocks farther away wait until the player comes closer.
- **Cheat:** `nanobots_free` toggles free nanobot building: no modules needed, and pastes are instant. It's for testing and creative play, and honours the world's Allow cheats setting.

## Capabilities

### New Capabilities
- `nanobot-modules`: Module items and tiers, installing them in the body, nanobot build speed and range, and the free-building cheat.

### Modified Capabilities
<!-- `blueprints` (from `add-blueprints`): pasting will need nanobot modules (unless the cheat is on) and build over time. Add a MODIFIED delta for it once `add-blueprints` is archived. -->

## Impact

- **Core:** module item kinds, body slots on the torso, a nanobot build queue processed in factory steps, and an energy cost on the robot's power source.
- **Client:** module slots in the Body tab, a build-queue indicator, and the shimmer on newly built blocks.
- **Cheats:** a `nanobots_free` console command.
- **Depends on** `add-blueprints` and later material tiers (ore spouts for Mk1, ruined reactors for Mk2 and Mk3).
