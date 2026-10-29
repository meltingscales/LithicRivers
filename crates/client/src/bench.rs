//! `--bench`: run benchmark scenarios headless and check them against the performance
//! budget (the `performance-budget` spec). Scenarios are ordinary scripts in
//! `scripts/bench/`; steps before the first `beat` are setup and aren't timed. Only the
//! timing reads real time, and it never feeds back into the game.

use crate::headless::{
    parse_script, validate, FrameSample, FrameTimes, Runner, DEFAULT_SIZE, EXIT_OK,
    EXIT_SCRIPT_ERROR,
};
use std::io::Write;
use std::time::Duration;

/// The most a tick or a frame may take.
pub const BUDGET: Duration = Duration::from_millis(16);

/// Exit status when a scenario is over budget.
pub const EXIT_OVER_BUDGET: i32 = 1;

/// Median, 95th percentile and worst of a set of times.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub median: Duration,
    pub p95: Duration,
    pub worst: Duration,
}

impl Stats {
    fn of(mut times: Vec<Duration>) -> Self {
        times.sort();
        let at = |q: f64| {
            if times.is_empty() {
                Duration::ZERO
            } else {
                times[((times.len() - 1) as f64 * q).round() as usize]
            }
        };
        Self {
            count: times.len(),
            median: at(0.5),
            p95: at(0.95),
            worst: times.last().copied().unwrap_or_default(),
        }
    }

    fn over(&self) -> bool {
        self.worst > BUDGET
    }
}

/// One scenario's tick and frame times.
#[derive(Debug, Clone, PartialEq)]
pub struct ScenarioResult {
    pub name: String,
    /// Input handling plus the client step, for frames where the game tick advanced
    pub ticks: Stats,
    /// The client step plus drawing, for every frame
    pub frames: Stats,
}

impl ScenarioResult {
    pub fn from_samples(name: &str, samples: &[FrameSample]) -> Self {
        Self {
            name: name.to_string(),
            ticks: Stats::of(
                samples
                    .iter()
                    .filter(|s| s.ticked)
                    .map(|s| s.input + s.step)
                    .collect(),
            ),
            frames: Stats::of(samples.iter().map(|s| s.step + s.draw).collect()),
        }
    }

    /// What went over budget, if anything.
    pub fn verdict(&self) -> &'static str {
        match (self.ticks.over(), self.frames.over()) {
            (false, false) => "ok",
            (true, false) => "OVER (ticks)",
            (false, true) => "OVER (frames)",
            (true, true) => "OVER (ticks and frames)",
        }
    }
}

/// Run one scenario script and collect its frame timings.
pub fn run_scenario(name: &str, text: &str) -> Result<ScenarioResult, String> {
    run_scenario_frames(name, text, false).map(|(result, _)| result)
}

/// Run a scenario, also returning each timed frame's screen when asked.
fn run_scenario_frames(
    name: &str,
    text: &str,
    keep_screens: bool,
) -> Result<(ScenarioResult, Vec<ratatui::buffer::Buffer>), String> {
    let script = parse_script(text).map_err(|e| format!("{name}: script error: {e}"))?;
    let mut runner = Runner::new(
        script.seed.unwrap_or(0),
        script.size.unwrap_or(DEFAULT_SIZE),
    );
    validate(&script, &runner.app).map_err(|e| format!("{name}: script error: {e}"))?;
    runner.bench = Some(FrameTimes {
        keep_screens,
        ..Default::default()
    });
    let report = runner.run_steps(&script.steps);
    if report.status != EXIT_OK {
        return Err(format!(
            "{name}: {}",
            report.error.unwrap_or_else(|| "script failed".into())
        ));
    }
    let times = runner.bench.take().unwrap_or_default();
    if times.frames.is_empty() {
        return Err(format!("{name}: no timed frames (is there a `beat` step?)"));
    }
    Ok((
        ScenarioResult::from_samples(name, &times.frames),
        times.screens,
    ))
}

/// The scenario whose frames `window-render` replays through the GPU renderer.
pub const WINDOW_RENDER_SOURCE: &str = "surface-walk";
/// The window-render scenario's screen size in cells.
const WINDOW_RENDER_SIZE: (u16, u16) = (80, 24);

/// `window-render`: the source scenario's frames drawn by the game window's renderer
/// into an offscreen texture, each frame timed until the GPU has finished it. `Ok(None)`
/// when no GPU adapter is available, as on CI runners.
pub fn window_render(text: &str) -> Result<Option<ScenarioResult>, String> {
    use crate::window::{fonts, settings::DEFAULT_FONT_PX};
    use ratatui_wgpu::{Builder, Dimensions, Font};
    use std::num::NonZeroU32;

    let (_, screens) = run_scenario_frames(WINDOW_RENDER_SOURCE, text, true)?;
    let (cell_w, cell_h) = fonts::cell_size(DEFAULT_FONT_PX);
    let (cols, rows) = WINDOW_RENDER_SIZE;
    let size = |cells: u16, px: u32| NonZeroU32::new(cells as u32 * px).expect("nonzero");
    let backend = futures_lite::future::block_on(
        Builder::<ratatui_wgpu::shaders::DefaultPostProcessor>::from_font(
            Font::new(fonts::MONO).expect("bundled font parses"),
        )
        .with_font_size_px(DEFAULT_FONT_PX)
        .with_width_and_height(Dimensions {
            width: size(cols, cell_w),
            height: size(rows, cell_h),
        })
        .build_headless(),
    );
    let Ok(backend) = backend else {
        return Ok(None);
    };
    let mut terminal = ratatui::Terminal::new(backend).map_err(|e| e.to_string())?;
    let mut samples = Vec::with_capacity(screens.len());
    for screen in &screens {
        let start = std::time::Instant::now();
        terminal
            .draw(|f| {
                let buf = f.buffer_mut();
                for y in 0..rows.min(screen.area.height) {
                    for x in 0..cols.min(screen.area.width) {
                        buf[(x, y)] = screen[(x, y)].clone();
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        terminal.backend().wait_for_gpu();
        samples.push(FrameSample {
            draw: start.elapsed(),
            ..Default::default()
        });
    }
    Ok(Some(ScenarioResult::from_samples(
        "window-render",
        &samples,
    )))
}

fn ms(d: Duration) -> String {
    format!("{:.2}", d.as_secs_f64() * 1000.0)
}

/// The report table for some results.
pub fn report(results: &[ScenarioResult]) -> String {
    let mut out = format!(
        "Budget: {} ms per tick and per frame ({} build, CPU: {})\n\n",
        BUDGET.as_millis(),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        },
        cpu_model()
    );
    out.push_str(&format!(
        "{:<20} {:>6} {:>8} {:>8} {:>8}   {:>6} {:>8} {:>8} {:>8}   result\n",
        "scenario", "ticks", "median", "p95", "worst", "frames", "median", "p95", "worst"
    ));
    for r in results {
        out.push_str(&format!(
            "{:<20} {:>6} {:>8} {:>8} {:>8}   {:>6} {:>8} {:>8} {:>8}   {}\n",
            r.name,
            r.ticks.count,
            ms(r.ticks.median),
            ms(r.ticks.p95),
            ms(r.ticks.worst),
            r.frames.count,
            ms(r.frames.median),
            ms(r.frames.p95),
            ms(r.frames.worst),
            r.verdict()
        ));
    }
    out.push_str("(times in ms)\n");
    out
}

/// The exit status for some results: over budget if any scenario is.
pub fn exit_status(results: &[ScenarioResult]) -> i32 {
    if results.iter().any(|r| r.verdict() != "ok") {
        EXIT_OVER_BUDGET
    } else {
        EXIT_OK
    }
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|info| {
            info.lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split_once(':'))
                .map(|(_, v)| v.trim().to_string())
        })
        .unwrap_or_else(|| "unknown".into())
}

/// Run every scenario file, print the table, and return the exit status.
pub fn run(paths: &[String], out: &mut impl Write, err_out: &mut impl Write) -> i32 {
    let mut results = Vec::new();
    for path in paths {
        let name = std::path::Path::new(path)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        let result = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {path}: {e}"))
            .and_then(|text| run_scenario(&name, &text));
        match result {
            Ok(r) => results.push(r),
            Err(e) => {
                let _ = writeln!(err_out, "{e}");
                return EXIT_SCRIPT_ERROR;
            }
        }
    }
    let mut skipped = String::new();
    let source = paths.iter().find(|p| {
        std::path::Path::new(p).file_stem().and_then(|s| s.to_str()) == Some(WINDOW_RENDER_SOURCE)
    });
    if let Some(path) = source {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string());
        match text.and_then(|t| window_render(&t)) {
            Ok(Some(r)) => results.push(r),
            Ok(None) => skipped = "window-render: skipped, no GPU adapter\n".to_string(),
            Err(e) => {
                let _ = writeln!(err_out, "window-render: {e}");
                return EXIT_SCRIPT_ERROR;
            }
        }
    }
    let _ = out.write_all(report(&results).as_bytes());
    let _ = out.write_all(skipped.as_bytes());
    exit_status(&results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(input: u64, step: u64, draw: u64, ticked: bool) -> FrameSample {
        FrameSample {
            input: Duration::from_millis(input),
            step: Duration::from_millis(step),
            draw: Duration::from_millis(draw),
            ticked,
        }
    }

    #[test]
    fn stats_and_verdicts() {
        let fast =
            ScenarioResult::from_samples("fast", &[sample(2, 1, 3, true), sample(0, 1, 4, false)]);
        assert_eq!(fast.ticks.count, 1);
        assert_eq!(fast.ticks.worst, Duration::from_millis(3));
        assert_eq!(fast.frames.count, 2);
        assert_eq!(fast.frames.worst, Duration::from_millis(5));
        assert_eq!(fast.verdict(), "ok");
        assert_eq!(exit_status(std::slice::from_ref(&fast)), EXIT_OK);

        // A 20 ms tick: input time counts toward ticks, not frames
        let slow_tick = ScenarioResult::from_samples("slow", &[sample(19, 1, 2, true)]);
        assert_eq!(slow_tick.verdict(), "OVER (ticks)");
        let slow_draw = ScenarioResult::from_samples("draw", &[sample(0, 1, 20, false)]);
        assert_eq!(slow_draw.verdict(), "OVER (frames)");
        assert_eq!(exit_status(&[fast, slow_draw]), EXIT_OVER_BUDGET);
    }

    #[test]
    fn report_lists_every_scenario() {
        let r = ScenarioResult::from_samples("surface-walk", &[sample(1, 1, 1, true)]);
        let table = report(&[r]);
        assert!(table.contains("surface-walk"));
        assert!(table.contains("Budget: 16 ms"));
        assert!(table.contains(" ok"));
    }

    /// The window renderer draws the source scenario's frames offscreen, or reports that
    /// there is no GPU adapter; either way without failing.
    #[test]
    fn window_render_runs_or_skips() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../scripts/bench/{WINDOW_RENDER_SOURCE}.txt"));
        let text = std::fs::read_to_string(path).unwrap();
        match window_render(&text).unwrap() {
            Some(r) => assert!(r.frames.count > 0 && r.ticks.count == 0),
            None => eprintln!("no GPU adapter: window-render skipped"),
        }
    }

    /// Every benchmark scenario must stay a valid script with timed frames.
    #[test]
    fn every_bench_scenario_runs() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/bench");
        let mut paths: Vec<_> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "txt"))
            .collect();
        paths.sort();
        assert!(paths.len() >= 4, "expected the four scenarios: {paths:?}");
        for path in paths {
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let text = std::fs::read_to_string(&path).unwrap();
            let result = run_scenario(&name, &text).unwrap_or_else(|e| panic!("{e}"));
            assert!(result.frames.count > 0, "{name} has no timed frames");
        }
    }
}
