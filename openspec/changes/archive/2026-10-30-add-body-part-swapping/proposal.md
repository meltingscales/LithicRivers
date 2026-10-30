# Proposal

## Why

The Body tab lists parts and their state, and offers a single action per replacement item: "[i] Install Salvaged SapienCorp Arm". It installs into the first missing part that fits, and that's all it can do. As more parts arrive (craftable heads, legs and fusion cores in `add-backup-bodies`, extra arm slots in `add-multi-arm-bodies`), the player needs to choose which part goes where, swap a worn part for a better one, and take a part out, from one place.

## What Changes

- **Pick a part:** the Body tab's part list becomes selectable with the menu keys and the mouse (`menu-navigation`, `mouse-menus`). The selected part shows its name, state, integrity and effect.
- **See what fits it:** a second list shows the replacement items in the inventory that fit the selected part (`replacement_parts`), with what each would install as.
- **Install, swap and remove:**
  - **Install** an item into a missing part, as today, but into the part the player chose.
  - **Swap:** installing into a part that isn't missing takes the old part out first.
  - **Remove** a part, leaving it missing. A Functional or Enhanced part comes out as its item, ready to install again. A Damaged part comes out as scrap, so removing and reinstalling never repairs it for free.
  - **The torso and the power source can't be removed**, since losing either kills the robot.
- **One key per action,** shown with its printable key: `body.INSTALL_PART` (now also swaps), and a new `body.REMOVE_PART`. Repair keeps working on the selected part.
- **Effects update at once:** the effects list shows the result of a change right away.

## Capabilities

### New Capabilities
- `body-parts`: Choosing a body part in the Body tab, and installing, swapping and removing parts.

### Modified Capabilities
<!-- arm-replacement-quest: its Install action is now this tab's install. -->

## Impact

- **Core:**
  - `Body::remove(part) -> Option<ItemKind>` and the swap through `Body::install`.
  - A table of what each part comes out as: its item when working, scrap when damaged.
- **Client:**
  - The Body tab's selectable part list and fitting-items list, with click targets.
  - `body.REMOVE_PART`, and `body.INSTALL_PART` acting on the selected part.
  - The repair window opening on the selected part.
- **Tests:**
  - core: install, swap and remove rules, including the locked torso and power source
  - the Body tab with both key sets and the mouse
  - save and reload after a swap
