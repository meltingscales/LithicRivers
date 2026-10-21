# Proposal

## Why

Building a factory is the main loop, and repeating a good layout by hand gets old: the tree farm, a furnace line, a power block. Factorio's blueprints let you copy a build and paste it anywhere. LithicRivers needs the same, and it fits the robot protagonist: later, nanobots in the robot's body will do the building (`add-nanobot-modules`).

## What Changes

- **Blueprint mode**, entered with a keybind on the World tab (the build modes also gain it as a fourth mode).
  - **Copy:** mark two corners of a rectangle. The factory blocks and placed blocks inside (with their facing, and filters, but not their contents) are saved as a blueprint.
  - **Paste:** a blinking preview of the blueprint follows the cursor, can be rotated, and is placed with the place key.
- **Replacing:** if a paste would cover player-built blocks (factory blocks or placed blocks; not grass, trees, rock or other terrain), a warning asks to confirm. Confirming breaks those blocks into the player's inventory first.
- **Cost:** pasting uses the blocks' items from the player's inventory, and the paste is refused, with a list of what's missing, if any are short. It needs no tool or module for now. `add-nanobot-modules` will add nanobot requirements and build speed later.
- **Blueprint library:** blueprints are saved with the world, can be named, and are listed in a Blueprints panel. One is selected at a time.

## Capabilities

### New Capabilities
- `blueprints`: Copying an area, the blueprint library, the blinking paste preview with rotation, the replace warning, and paying for pastes from the inventory.

### Modified Capabilities
<!-- None. Build modes gain a fourth mode; the existing three are unchanged. -->

## Impact

- **Core:** a `Blueprint { size, blocks: Vec<(offset, kind, facing, filter)> }` type, `blueprints::capture` and `blueprints::paste` (all-or-nothing, through `factory::place`), and a library on `Resources` (saved).
- **Client:** a Blueprint build mode, the corner selection and blinking preview (on `app.clock`), the replace confirmation, and the Blueprints panel.
- **Depends on** factory blocks and the build modes.
