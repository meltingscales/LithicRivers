# Proposal

## Why

The robot starts with its left arm missing, and `docs/STEAM-MVP.md` plans the second quest around it: leave the bunker, travel about a mile to the abandoned SapienCorp facility, and find a replacement arm in a guaranteed chest near its assembly line. The pieces are half there:
- The facility structure is placed at (200, 1609) when a world is created, with a quest marker.
- Its legend has a quest-treasure tile (`1`), but none is placed, and nothing spawns from that tile kind.
- Repairs can't restore a missing part, so today there is no way to get the arm back at all.
- Nothing gives the quest, tracks it or completes it.

This quest is the player's first long goal after the tutorial, so the Steam MVP needs it to work end to end.

## What Changes

- **The quest, "A New Arm":** the broken android gives it once its own repair quest is done. It points the player to the facility and explains that SapienCorp built arms like the robot's there.
  - **Objectives:**
    1. reach the facility
    2. find the salvaged arm
    3. install it
  - The Quests tab shows them, as with the tutorial quest.
- **A guaranteed arm chest:** the facility's assembly-line level gets a quest-treasure tile. When the structure is placed, the tile becomes a chest holding one **Salvaged SapienCorp Arm** (a new item). It is placed exactly once per world, so the arm can't be farmed.
- **Installing a part:** in the Body tab, a missing part with a matching replacement in the inventory can be installed. The item is used up, and the part becomes Functional at 70 integrity. Repairs then work on it as on any other part.
- **The reward:** completing the quest gives the arm back and a short closing line from the android. Later quests can build on the robot being whole.

## Capabilities

### New Capabilities
- `arm-replacement-quest`: The quest, the facility's arm chest, the salvaged arm item, and installing a replacement into a missing part.

### Modified Capabilities
<!-- None: the quest system's states and objective tracking are reused. -->

## Impact

- **Core:**
  - `QuestType::ReplaceArm`, and an `InstallPart` objective type.
  - `ItemKind::SalvagedArm`, and the quest-treasure tile spawning the chest.
  - `Body::install(part, item)`.
  - The android's dialogue after its repair quest.
- **Content:** a `1` tile in `sapiencorp-factory.lrstructure`'s assembly-line level, near the line, past its enemies.
- **Client:** an "Install" action in the Body tab for missing parts, and sprites for the arm item and the quest chest.
- **Tests:**
  - the quest flows from the android to completion
  - the chest appears once, holding the arm
  - installing restores the part, and a second install is refused
  - save and reload at every step
