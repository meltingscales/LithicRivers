//! ASCII bar graphs for the factory stats: the pole box's power graph and the Production
//! tab's per-minute graph. Pure functions, so they are unit tested.

/// Block characters for a bar's top cell, from an eighth to full.
const EIGHTHS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// A rendered graph: `rows` top to bottom, each `width` characters, and for each column
/// whether it should be drawn as a warning (a second when the network was short).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph {
    pub rows: Vec<String>,
    pub warn: Vec<bool>,
}

/// Bars for `values`, one column each, newest on the right, scaled so `scale` fills
/// `height` rows. With more values than `width`, values are merged into `width` buckets
/// (their largest value, warning if any was). With fewer, the graph is right-aligned and
/// padded with blanks on the left.
pub fn bars(values: &[(u64, bool)], scale: u64, width: usize, height: usize) -> Graph {
    let columns: Vec<(u64, bool)> = if values.len() > width && width > 0 {
        (0..width)
            .map(|c| {
                let start = c * values.len() / width;
                let end = ((c + 1) * values.len() / width).max(start + 1);
                let bucket = &values[start..end];
                (
                    bucket.iter().map(|&(v, _)| v).max().unwrap_or(0),
                    bucket.iter().any(|&(_, w)| w),
                )
            })
            .collect()
    } else {
        values.to_vec()
    };
    let pad = width.saturating_sub(columns.len());
    let scale = scale.max(1);
    // Each column's height in eighths of a row
    let eighths: Vec<u64> = columns
        .iter()
        .map(|&(v, _)| (v.min(scale) * height as u64 * 8).div_ceil(scale))
        .collect();
    let rows = (0..height)
        .map(|row| {
            // Rows count up from the bottom
            let floor = (height - 1 - row) as u64 * 8;
            let mut line: String = " ".repeat(pad);
            for &e in &eighths {
                line.push(if e >= floor + 8 {
                    '█'
                } else if e > floor {
                    EIGHTHS[(e - floor - 1) as usize]
                } else {
                    ' '
                });
            }
            line
        })
        .collect();
    let mut warn = vec![false; pad];
    warn.extend(columns.iter().map(|&(_, w)| w));
    Graph { rows, warn }
}

/// The power graph: demand per sample, scaled to the larger of supply and demand over
/// the samples shown, with short samples marked.
pub fn power(
    samples: &[lithicrivers_core::factory_stats::PowerSample],
    width: usize,
    height: usize,
) -> Graph {
    let scale = samples
        .iter()
        .map(|s| s.supply_w.max(s.demand_w))
        .max()
        .unwrap_or(0);
    let values: Vec<(u64, bool)> = samples.iter().map(|s| (s.demand_w, s.short())).collect();
    bars(&values, scale, width, height)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lithicrivers_core::factory_stats::PowerSample;

    #[test]
    fn bars_scale_to_the_largest_value_and_fill_from_the_bottom() {
        let g = bars(&[(0, false), (4, false), (8, false), (2, false)], 8, 4, 2);
        assert_eq!(g.rows, ["  █ ".to_string(), " ██▄".to_string()]);
        assert_eq!(g.warn, [false; 4]);
    }

    #[test]
    fn short_samples_are_marked() {
        let samples = [
            PowerSample {
                supply_w: 90,
                demand_w: 90,
            },
            PowerSample {
                supply_w: 90,
                demand_w: 180,
            },
        ];
        let g = power(&samples, 2, 1);
        assert_eq!(g.rows, ["▄█".to_string()]);
        assert_eq!(g.warn, [false, true]);
    }

    #[test]
    fn graphs_are_always_the_width_asked_for() {
        // Fewer values: right-aligned
        let g = bars(&[(1, true)], 1, 5, 1);
        assert_eq!(g.rows, ["    █".to_string()]);
        assert_eq!(g.warn, [false, false, false, false, true]);
        // More values: merged into buckets of their largest, warning if any warned
        let values: Vec<(u64, bool)> = (0..10).map(|v| (v, v == 3)).collect();
        let g = bars(&values, 9, 5, 1);
        assert_eq!(g.rows[0].chars().count(), 5);
        assert_eq!(g.warn, [false, true, false, false, false]);
        assert_eq!(g.rows[0].chars().last(), Some('█'));
        // Empty and zero-height graphs don't panic
        assert_eq!(bars(&[], 0, 3, 1).rows, ["   ".to_string()]);
        assert!(bars(&[(1, false)], 1, 3, 0).rows.is_empty());
    }
}
