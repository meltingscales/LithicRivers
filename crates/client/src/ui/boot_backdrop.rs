//! The backdrop behind the System Boot log: a wall of old, cracked metal panels
//! under a dying monitor's light (a scanline sweep, static, a blinking fault lamp
//! on broken panels and the odd horizontal tear).
//!
//! Everything reads from the clock's animation milliseconds and a hash of cell
//! positions, so a headless run draws the same frames every time.

use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

/// Panel size in cells, border included.
const TILE_W: u16 = 8;
const TILE_H: u16 = 4;

const LINE: Color = Color::Rgb(46, 52, 60);
const GRIT: Color = Color::Rgb(32, 36, 42);
const RUBBLE: Color = Color::Rgb(58, 54, 50);
const SCANLINE: Color = Color::Rgb(78, 104, 128);
const STATIC: Color = Color::Rgb(110, 110, 110);
const LAMP: Color = Color::Rgb(190, 70, 40);

/// Milliseconds the scanline spends on each row.
const SCANLINE_ROW_MS: u64 = 60;
/// A tear happens once per period, for `TEAR_MS`.
const TEAR_PERIOD_MS: u64 = 4000;
const TEAR_MS: u64 = 250;
/// Static re-rolls every frame of this length.
const STATIC_FRAME_MS: u64 = 100;

fn hash(a: u64, b: u64, c: u64) -> u64 {
    let mut z = a
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(b.wrapping_mul(0xBF58_476D_1CE4_E5B9))
        .wrapping_add(c.wrapping_mul(0x94D0_49BB_1331_11EB));
    z ^= z >> 30;
    z = z.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z ^= z >> 27;
    z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[derive(Clone, Copy, PartialEq)]
enum Panel {
    Intact,
    Cracked,
    Missing,
}

fn panel(tx: u64, ty: u64) -> Panel {
    match hash(tx, ty, 1) % 100 {
        0..=7 => Panel::Missing,
        8..=29 => Panel::Cracked,
        _ => Panel::Intact,
    }
}

/// The still wall at a cell: its glyph and color.
fn wall(x: u64, y: u64) -> (char, Color) {
    let (tx, ty) = (x / TILE_W as u64, y / TILE_H as u64);
    let (lx, ly) = (x % TILE_W as u64, y % TILE_H as u64);
    let kind = panel(tx, ty);
    let roll = hash(x, y, 2) % 100;
    if kind == Panel::Missing {
        // A hole in the wall: dark, with a little dust
        return if roll < 6 { ('.', GRIT) } else { (' ', GRIT) };
    }
    let seam = match (lx == 0, ly == 0) {
        (true, true) => Some('┼'),
        (false, true) => Some('─'),
        (true, false) => Some('│'),
        _ => None,
    };
    if let Some(seam) = seam {
        // Broken panels have lost bits of their seams
        if kind == Panel::Cracked && roll < 30 {
            return (' ', LINE);
        }
        return (seam, LINE);
    }
    match kind {
        Panel::Cracked if roll < 35 => (if roll < 12 { '▒' } else { '░' }, RUBBLE),
        _ if (lx == 1 || lx == TILE_W as u64 - 1) && (ly == 1 || ly == TILE_H as u64 - 1) => {
            ('·', LINE)
        }
        _ if roll < 10 => ('░', GRIT),
        _ => (' ', GRIT),
    }
}

/// The wall with its animations at a moment.
pub(crate) struct BootBackdrop {
    pub millis: u64,
}

impl Widget for BootBackdrop {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let ms = self.millis;
        let height = area.height as u64;
        // The scanline sweeps down, then rests off screen for a moment
        let scan_row = (ms / SCANLINE_ROW_MS) % (height + 20);
        // A tear shifts a three-row band sideways
        let tear = (ms % TEAR_PERIOD_MS < TEAR_MS).then(|| {
            let n = ms / TEAR_PERIOD_MS;
            (hash(n, 0, 3) % height.max(1), 2 + hash(n, 0, 4) % 5)
        });
        let static_frame = ms / STATIC_FRAME_MS;

        for y in area.top()..area.bottom() {
            let row = (y - area.top()) as u64;
            let shift = match tear {
                Some((top, by)) if row >= top && row < top + 3 => by,
                _ => 0,
            };
            for x in area.left()..area.right() {
                let col = (x - area.left()) as u64;
                let wx = col + shift;
                let (mut ch, mut fg) = wall(wx, row);

                // A fault lamp blinks in one corner of each broken panel
                let (tx, ty) = (wx / TILE_W as u64, row / TILE_H as u64);
                if panel(tx, ty) == Panel::Cracked
                    && wx % TILE_W as u64 == 2
                    && row % TILE_H as u64 == 2
                {
                    let period = 300 + hash(tx, ty, 5) % 900;
                    if (ms / period) % 2 == 0 {
                        (ch, fg) = ('•', LAMP);
                    }
                }

                if hash(col, row, static_frame) % 1000 < 5 {
                    ch = ['░', '▒', '▓', ':'][(hash(col, row, static_frame + 1) % 4) as usize];
                    fg = STATIC;
                }

                if row == scan_row || row + 1 == scan_row {
                    fg = SCANLINE;
                }

                if let Some(cell) = buf.cell_mut((x, y)) {
                    cell.set_char(ch).set_fg(fg);
                }
            }
        }
    }
}
