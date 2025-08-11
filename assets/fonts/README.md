LithicRivers font assets

This directory contains font assets used by the Bevy client.

Primary (monospace) font
- Expected path: assets/fonts/monospace.ttf
- Purpose: ASCII-style 2D renderer uses a monospaced font for tile glyphs.
- Recommended options (OFL/Apache licensed):
  - JetBrains Mono: https://www.jetbrains.com/lp/mono/
  - Fira Mono: https://github.com/mozilla/Fira
  - DejaVu Sans Mono: https://dejavu-fonts.github.io/

Fallback font
- If `assets/fonts/monospace.ttf` is missing or not yet loaded, the client will fall back to Bevy’s bundled `fonts/FiraSans-Bold.ttf`.

Instructions
1) Place your chosen monospaced TTF at:
   assets/fonts/monospace.ttf
2) Run the client (make client). The ASCII renderer will use the monospaced font when available, otherwise the fallback.

Notes
- Keep the font filename as `monospace.ttf` to avoid changing code.
- Prefer truly monospaced fonts to ensure glyph alignment.
