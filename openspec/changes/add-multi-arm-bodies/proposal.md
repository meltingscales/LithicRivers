# Proposal

## Why

The user wants quad, hexa and octa-armed builds: robots with four, six or eight arms that can wield that many guns and fire them all in one turn (`add-firearms`), woken from an end-game resurrection vat (`add-backup-bodies`). Today a body has exactly two arm slots (`BodyPartType::LeftArm` and `RightArm`), so both of those changes need a body model that can hold more. That shared piece is this change.

## What Changes

- **Body layouts:** a body has a layout with 2, 4, 6 or 8 arm slots. Every existing body, and every new game, has the 2-arm layout, so nothing changes for them.
- **Extra arm slots** beyond the left and right arm: arms 3 to 8. Each slot holds an arm part, can be Missing, Damaged, Functional or Enhanced, can be repaired and installed into, and holds one weapon (`add-firearms`).
- **How a body gets more arms:** the layout is part of the torso.
  - A multi-arm torso (quad, hexa or octa) is an end-game part.
  - A robot gets a multi-arm body by waking in an advanced vat built with one (`add-backup-bodies`). A living robot doesn't swap torsos.
- **What extra arms do:**
  - one weapon slot each
  - mining and melee use the two best working arms (`body-damage-effects` spec), so extra arms don't stack mining speed, but losing two arms hurts less
- **Display:**
  - The Body tab lists every arm slot.
  - The body art shows the extra arms.
  - The look panel and combat panel count the arms.

## Capabilities

### New Capabilities
- `multi-arm-bodies`: Body layouts with 2, 4, 6 or 8 arms, extra arm slots, multi-arm torsos, and what extra arms do.

### Modified Capabilities
<!-- None in openspec/specs: bodies have no spec yet. -->

## Impact

- **Core:**
  - `BodyPartType` gains `ExtraArm(u8)` for arms 3 to 8. `LeftArm` and `RightArm` stay, so old saves load unchanged.
  - `Body::layout()` comes from the torso part's kind.
  - Damage targeting includes extra arms.
  - Repair and install work on them.
- **Client:** the Body tab and body art (`body_art()` gains layouts for 4, 6 and 8 arms).
- **Saves:** bodies serialize their parts by type; `ExtraArm(n)` is new, and old saves have none.
- **Needed by:** `add-firearms` (more than two weapons) and `add-backup-bodies` (the advanced vat).
