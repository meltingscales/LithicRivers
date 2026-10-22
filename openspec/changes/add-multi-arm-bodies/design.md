# Design

## Context

- **`Body { parts: HashMap<BodyPartType, BodyPart> }`**, with seven part types: Head, Torso, PowerSource, LeftArm, RightArm, LeftLeg, RightLeg.
- **Damage** picks a random part that isn't Missing, in sorted order for determinism.
- **The body panel** draws a 13x13 schematic from `body_art()`, with a marker character per part.

## Goals / Non-Goals

**Goals:**
- Room for 4, 6 or 8 arms without breaking two-armed bodies or old saves.
- Deterministic damage over every part.
- Extra arms that matter, but mostly through weapons.

**Non-Goals:**
- Extra legs or heads.
- Swapping torsos on a living robot.
- Per-arm animations.

## Decisions

### Representation
- `BodyPartType::ExtraArm(u8)` with n from 3 to 8. It derives Ord, so damage targeting stays sorted and deterministic. LeftArm and RightArm remain arms 1 and 2.
- **The layout comes from the torso:** a `Torso` part gets a `torso_kind` (Standard, Quad, Hexa, Octa), saved with a serde default of Standard. A quad torso's body has ExtraArm(3) and ExtraArm(4), and so on.
- **Arm helpers:** `Body::arms()` iterates every arm slot in order, and every arm-based rule (weapon slots, the arm effects in `add-body-damage-effects`) goes through it.

### What extra arms do
- **Weapons:** one slot per working arm (`add-firearms`).
- **Mining and melee:** the two best working arms, so a four-armed robot mines like a two-armed one at full health, but loses less when two arms are damaged.

### Art
`body_art()` gets one schematic per layout. Extra arms draw as additional `A`/`a` markers stacked down the sides of the torso, keeping 13 by 13.

## Risks / Trade-offs

- [Every arm rule has to remember extra arms] → Everything goes through `Body::arms()`, and a test checks a quad body under each arm rule.
- [Save size] → At most six extra parts per body.
