//! The fonts the game window draws with. They ship with the game, so the window looks
//! the same on every machine (`game-window` spec).

/// DejaVu Sans Mono: box drawing, block elements and most symbols, in one monospace face.
pub const MONO: &[u8] = include_bytes!("../../assets/fonts/DejaVuSansMono.ttf");

/// Every bundled font, in fallback order. A fallback font must not be narrower than
/// `MONO`: the renderer sizes cells to the narrowest font.
pub const ALL: &[&[u8]] = &[MONO];

/// Whether some bundled font has a glyph for `c`.
pub fn covers(c: char) -> bool {
    ALL.iter().any(|data| {
        ttf_parser::Face::parse(data, 0)
            .ok()
            .and_then(|face| face.glyph_index(c))
            .is_some()
    })
}

/// The width in pixels one font gives a cell at a height, computed as the renderer does:
/// the advance of `m` (or of the missing-glyph shape) scaled to the height.
fn char_width(data: &[u8], height_px: u32) -> u32 {
    let face = ttf_parser::Face::parse(data, 0).expect("bundled font parses");
    let glyph = face.glyph_index('m').unwrap_or_default();
    let advance = face.glyph_hor_advance(glyph).unwrap_or_default() as f32;
    (advance * height_px as f32 / face.height() as f32) as u32
}

/// The size of one screen cell in pixels for a font height. The renderer makes cells as
/// wide as the narrowest bundled font, so this matches its layout exactly.
pub fn cell_size(height_px: u32) -> (u32, u32) {
    let width = ALL.iter().map(|data| char_width(data, height_px)).min();
    (width.unwrap_or_default().max(1), height_px.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    /// Every character in files the game draws from: sprites and other assets, and the
    /// string literals in client and core sources (comment lines skipped).
    fn drawn_characters() -> BTreeSet<(char, String)> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files = Vec::new();
        collect(&root.join("assets/sprites"), &mut files);
        collect(&root.join("src"), &mut files);
        collect(&root.join("../core/src"), &mut files);
        let mut found = BTreeSet::new();
        for path in files {
            let skip_dir = path.components().any(|c| c.as_os_str() == "bin");
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };
            if skip_dir || !["rs", "txt", "json", "toml"].contains(&ext) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            for line in text.lines() {
                let trimmed = line.trim_start();
                if ext == "rs" && (trimmed.starts_with("//") || trimmed.starts_with("#[")) {
                    continue;
                }
                for c in line.chars().filter(|c| !c.is_ascii() && !c.is_control()) {
                    found.insert((c, path.display().to_string()));
                }
            }
        }
        found
    }

    fn collect(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect(&path, out);
            } else {
                out.push(path);
            }
        }
    }

    #[test]
    fn bundled_fonts_cover_every_drawn_character() {
        for c in ' '..='~' {
            assert!(covers(c), "ASCII {c:?} missing");
        }
        let mut missing: Vec<String> = drawn_characters()
            .into_iter()
            .filter(|(c, _)| !covers(*c))
            .map(|(c, file)| format!("{c:?} U+{:04X} in {file}", c as u32))
            .collect();
        missing.dedup();
        assert!(
            missing.is_empty(),
            "missing glyphs:\n{}",
            missing.join("\n")
        );
    }

    #[test]
    fn no_font_narrows_cells() {
        for height in [12, 18, 24, 32] {
            assert_eq!(
                cell_size(height).0,
                char_width(MONO, height),
                "at {height}px"
            );
        }
    }

    #[test]
    fn cells_are_taller_than_wide() {
        let (w, h) = cell_size(18);
        assert!(w > 0 && w < h, "{w}x{h}");
    }
}
