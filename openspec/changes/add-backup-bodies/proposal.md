# Proposal

## Why

`add-player-death` respawns the robot at the bunker with the default body: a stand-in until backup bodies were designed. The user's idea (IDEAS-INBOX.md) fits the robot protagonist and the factory loop. Prepare a spare body in advance, built from crafted parts and kept in a Robot Resurrection Vat, and wake up in it. Dying without one should feel like a mistake to fix, not a free reset.

## What Changes

- **Craftable body parts:** a head, torso, fusion core (power source), left and right arm, and left and right leg, each with a recipe from the factory tiers. They install into missing parts of the living robot too (`add-arm-replacement-quest`'s install action).
- **Robot Resurrection Vat:** a powered factory block.
  - It holds one spare body, assembled from the parts put into its slots.
  - Its window shows which parts are in and their state.
  - It needs power to keep the spare ready; an unpowered vat shows "dormant" and can't be chosen.
- **The Resurrect screen** replaces the system-failure screen's respawn choice:
  - It lists every ready vat, with its spare body's parts, energy and distance, and the player picks one.
  - The spare body is used up. The vat stays, empty, to be filled again.
- **Without a ready vat:** the robot wakes in a **junker body** at the SapienCorp HQ, every part damaged, with the thought "I should really make a vat...".
- **The respawn flyover:** after choosing, the view zooms out, flies across the world to the chosen vat or the HQ, zooms back in to zoom level 3, and the robot wakes. Any key skips it. It's only drawn, so the game's state never depends on it, and it runs on the game clock, so scripts replay it exactly.

## Capabilities

### New Capabilities
- `backup-bodies`: Craftable body parts, the resurrection vat, the Resurrect screen, the junker fallback and the respawn flyover.

### Modified Capabilities
- `player-death`: respawning chooses a vat or falls back to the junker body, instead of the default body at the bunker.

## Impact

- **Core:**
  - Body-part items and recipes.
  - `FactoryKind::ResurrectionVat` with part slots and power use.
  - `death::resurrect(vat)` and the junker fallback.
  - The HQ position, stored when a new world places the facility.
- **Client:**
  - The vat window.
  - The Resurrect screen (a list with each spare body's state).
  - The flyover, a camera path on `app.clock` that any key skips.
- **Showcase:** a powered vat with a full spare body.
- **Depends on:**
  - `add-player-death`
  - `add-arm-replacement-quest`'s install action
  - the power grid

## Open question

- What a vat costs, and which factory tier its parts come from: for the user to decide before tasks start.
