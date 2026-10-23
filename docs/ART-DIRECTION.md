# Art direction

LithicRivers draws everything in text: a 1x glyph on the map (2x and 3x when zoomed) and 12x8 art in the look panel and item views. Each sprite has one color, set in its `data.json`. Three references set the look:

- **Caves of Qud:**
  - *The glyph is the thing.* Every tile, creature and item reads from one character and one strong color.
  - *Specific, earthy palettes.* Each material gets its own color: rust, bone, moss, char, verdigris. There are no generic grays when a material has a real color.
  - *Ancient and strange.* Objects look old, salvaged, grown or half-understood. Descriptions are short and specific, and give a hint of history.
- **Metroid (Fusion, Zero Mission, Super):**
  - *Ruins of a vanished, advanced civilization*, explored alone. SapienCorp tech should feel like that: plated, chunky, built to last, now broken.
  - *Biomechanical forms.* Machines are made of panels, vents, pipes and cores. Living things and machines blur, as with the robot protagonist and the androids.
  - *Glowing focal points.* A lit core, an eye or a furnace mouth stands out against dark plating. Use bright characters (`@`, `*`, `o`, `█`) for the one part that should catch the eye.
- **CICADAMATA:** in one reviewer's words, a minimalist, low-poly abstraction of *Marathon*'s look.
  - *Bold, simple shapes.* Reduce a thing to its silhouette and one or two details. Leave empty space empty.
  - *Readable at speed.* A sprite must still read at a glance in a fight or a busy factory.

## Rules for sprites

- **Characters:** ASCII, CP437-style box drawing (`─│┌┐└┘├┤┬┴┼═║╔╗╚╝`), blocks and shades (`█▓▒░▀▄▌▐`), and simple symbols. No emoji, nothing double-width.
- **12x8 art:** exactly 12 columns by 8 rows. One centered subject, with negative space around it, no frame, no text labels. The silhouette comes first, then one or two details, then a focal point.
- **Facings:** factory blocks with a facing get art and small sprites per facing. The output side carries the arrow or the opening.
- **1x glyphs:** blocks that appear side by side in normal play must differ by glyph or by color: a belt and a harvester, dirt and a sapling, a furnace and a feral dog.
- **Blocks and dropped items:** a placed block and the same item lying on the ground must never look the same on the map, in any facing or lit state. A subtle difference is enough: by default the item is a darker shade of the block's color (about 75% brightness). `blocks_and_dropped_items_look_different` in `sprite_loader.rs` checks this.
- **Color:** one hex color per sprite. Lit or active states get their own sprite with a hotter color (`crude_furnace_lit_*`).
- **Descriptions:** one short sentence, specific and a little weird, in the Caves of Qud way. Say what the thing is or does, and add a hint of history when it fits.

Mechanics are borrowed from other games, but the content (blocks, materials, lore and art) is LithicRivers' own.
