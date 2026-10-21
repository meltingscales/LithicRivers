# Design

## Context

- A sprite is a folder `<category>/<name>.lrsprite` with `data.json` (name, color, description), `sprites.txt` (1x, 2x and 3x glyphs) and `art12x8.txt` (12 columns by 8 rows).
- The sprite loader checks that every line of a size has the same width.

## Goals / Non-Goals

**Goals:** every sprite has art that reads at a glance in one color; a check that keeps it that way.

**Non-Goals:** multi-color art, animation, new sprites.

## Decisions

- **Placeholder detection:** art is a placeholder if more than half of its characters are `x`. The test names every placeholder sprite.
- **Style:** ASCII and box-drawing characters only, centered, and readable in the sprite's single `data.json` color. Factory blocks with a facing get one art file per facing, as their small sprites do.
- **Gallery:** a headless-only debug screen lists every sprite with its name, small sizes and art. A snapshot script records it, so art changes appear as snapshot diffs.
- **Order:** factory blocks and items first (they're the ones players inspect most), then entities, tiles and fluids, as separate tasks so the work can land in parts.

## Risks / Trade-offs

- [Art is subjective] → Each batch ends with a user check of the gallery snapshot.
