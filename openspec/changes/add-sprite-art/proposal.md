# Proposal

## Why

Every one of the 73 sprites has placeholder 12x8 art: a box of `x` with the sprite's name typed in the middle. The look panel, and soon the Item Dictionary, show this art, and a Steam release can't ship with it. Some small sprites also read poorly (for example, an auto-harvester and a belt both draw an arrow at 1x).

## What Changes

- **Real 12x8 art** for every sprite: entities, factory blocks, fluids, items and tiles.
- **A review of the 1x, 2x and 3x sprites** for ones that are hard to tell apart or don't fit their color, fixing the worst.
- **A test that fails on placeholder art**, so new sprites can't ship without art.
- **A headless gallery script** that renders every sprite at every size into a snapshot, so art changes show up in review.

## Capabilities

### New Capabilities
- `sprite-art`: Every sprite has real art; the placeholder check and the gallery snapshot.

### Modified Capabilities
<!-- None -->

## Impact

- **Assets:** `crates/client/assets/sprites/**/art12x8.txt` and some `sprites.txt`.
- **Client:** the sprite-loader test for placeholder art, and the gallery (a debug panel or headless step) for the snapshot.
- **No gameplay changes.**
