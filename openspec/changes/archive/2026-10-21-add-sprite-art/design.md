# Design

## Context

- A sprite is a folder `<category>/<name>.lrsprite` with `data.json` (name, color, description), `sprites.txt` (1x, 2x and 3x glyphs) and `art12x8.txt` (12 columns by 8 rows).
- The sprite loader checks that every line of a size has the same width.

## Goals / Non-Goals

**Goals:** every sprite has art that reads at a glance in one color, following `docs/ART-DIRECTION.md` (Caves of Qud, Metroid, CICADAMATA); a check that keeps it that way.

**Non-Goals:** multi-color art, animation, new sprites.

## Decisions

- **Placeholder detection:** art is a placeholder if more than half of its characters are `x`. The test names every placeholder sprite.
- **Style:** ASCII and box-drawing characters only, centered, and readable in the sprite's single `data.json` color. Factory blocks with a facing get one art file per facing, as their small sprites do.
- **Gallery:** a headless-only debug screen lists every sprite with its name, small sizes and art. A snapshot script records it, so art changes appear as snapshot diffs.
- **Order:** factory blocks and items first (they're the ones players inspect most), then entities, tiles and fluids, as separate tasks so the work can land in parts.

## Risks / Trade-offs

- [Art is subjective] → Each batch ends with a user check of the gallery snapshot.

## Implementation Notes

- **Art direction:** chosen by the user and recorded in `docs/ART-DIRECTION.md`, linked from AGENTS.md's guiding principles and from `docs/INSPIRATION.md`. Caves of Qud gives one glyph and one material color per object; Metroid gives plated, biomechanical SapienCorp tech with one glowing focal point; CICADAMATA gives bold, minimal silhouettes that read at speed.
- **Width checks count characters, not bytes.** The loader now validates 1x/2x/3x sprites and 12x8 art with `chars().count()`, so CP437-style box drawing and block characters (`═║▓▒░▄▀█`) count as one column.
- **Gallery:** a unit test, `sprite_loader::art_tests::sprite_gallery`, instead of a headless debug screen. It snapshots every sprite (color, description, 1x/2x/3x and every art entry, including mood portraits) to `crates/client/src/snapshots/sprite_gallery.snap`. The name is fixed because `utility_sprite_demo` also compiles `sprite_loader.rs`.
- **Placeholder check:** `no_sprite_has_placeholder_art` (more than half `x`). All 91 art files (83 sprites, plus the player's and android's mood portraits) now have art.
- **Look-alike 1x glyphs fixed:**
  - player `*` to `@` (it clashed with rock, biochar and acid)
  - feral dog `f` to `d` (it clashed with the furnace)
  - sapling `,` to `τ` (it clashed with dirt)
  - stone `s` to `■`, string `s` to `∞`, and wood shavings `s` to `"`

  World snapshots changed only by these swaps; portrait and item-art panels now show real art.
- **Worldgen marker tiles** (spawns, scrap and treasure markers) are never seen in play and are drawn as dashed boxes with a symbol.
