//! The attract-mode demo: short scenario scripts from `assets/demos/` played one after
//! another in real game worlds, under a scrolling DEMO banner.
//!
//! A scenario is an ordinary headless script. Its steps up to the first `beat` set the scene
//! at once; after that, one step plays per beat (a `wait` holds for its time). Scripted keys
//! go through the same input dispatcher as the player's, and everything runs on `app.clock`,
//! so the demo is deterministic.

use crate::headless::{parse_script, Step};
use crate::{App, EmbeddedAssets, SplashState};
use crossterm::event::KeyCode;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Scenario files, in playing order.
pub const SCENARIOS: &[&str] = &[
    "explore",
    "npc-dialogue",
    "mining-crafting",
    "combat",
    "dog-arena",
    "building",
    "blueprints",
    "layers",
    "factory",
    "tree-farm",
];
/// A scenario ends after this long even if its script has more steps.
pub const SCENARIO_MAX: Duration = Duration::from_secs(30);
/// Playback runs this much slower than a script's `beat` and `wait` times, as a fraction
/// (3/2: one and a half times as slow), so viewers can follow each interaction.
pub const SLOWDOWN: (u64, u64) = (3, 2);
/// The blank title card between scenarios.
pub const CUT: Duration = Duration::from_millis(900);
/// Time per step until a scenario sets its own `beat`.
pub const DEFAULT_BEAT_MS: u64 = 200;

/// A script time in ms, slowed for playback.
pub fn playback(ms: u64) -> Duration {
    Duration::from_millis(ms * SLOWDOWN.0 / SLOWDOWN.1)
}

pub struct Scenario {
    /// The script's first comment line
    pub name: String,
    pub steps: Vec<Step>,
}

/// The text of a scenario file.
pub fn scenario_text(file: &str) -> String {
    let path = format!("demos/{file}.txt");
    let asset = EmbeddedAssets::get(&path).unwrap_or_else(|| panic!("missing {path}"));
    String::from_utf8_lossy(&asset.data).replace("\r\n", "\n")
}

/// Every scenario, in order. Panics on a script that doesn't parse: the scenario tests run
/// each one, so a broken script fails `cargo test` before it ships.
pub fn scenarios() -> Vec<Scenario> {
    SCENARIOS
        .iter()
        .map(|file| {
            let text = scenario_text(file);
            let name = text
                .lines()
                .find_map(|l| l.strip_prefix("# "))
                .unwrap_or(file)
                .trim()
                .to_string();
            let script = parse_script(&text).unwrap_or_else(|e| panic!("demo {file}: {e}"));
            Scenario {
                name,
                steps: script.steps.into_iter().map(|l| l.step).collect(),
            }
        })
        .collect()
}

pub struct DemoState {
    pub scenarios: Vec<Scenario>,
    pub index: usize,
    /// The next step of the current scenario
    pub cursor: usize,
    pub beat: Duration,
    /// When the next beat plays
    pub next_at: Instant,
    /// When the current scenario started
    pub scenario_started: Instant,
    /// When the demo started (the banner scrolls from here)
    pub started: Instant,
    /// While set, the title card for the current scenario shows until then
    pub cut_until: Option<Instant>,
    /// Keys left to press from a `key` step with a count, or a `type` step
    pending: VecDeque<KeyCode>,
}

impl DemoState {
    pub fn current_name(&self) -> &str {
        &self.scenarios[self.index].name
    }
}

impl App {
    /// Start the demo from its first scenario, with a title card.
    pub(crate) fn start_demo(&mut self) {
        let now = self.clock.now();
        self.demo = Some(DemoState {
            scenarios: scenarios(),
            index: 0,
            cursor: 0,
            beat: playback(DEFAULT_BEAT_MS),
            next_at: now,
            scenario_started: now,
            started: now,
            cut_until: Some(now + CUT),
            pending: VecDeque::new(),
        });
    }

    /// End the demo: drop its world and go back to the title screen with Watch Demo
    /// selected. Nothing was saved.
    pub(crate) fn stop_demo(&mut self) {
        self.demo = None;
        self.current_world = None;
        self.core.game = lithicrivers_core::game::Game::new_empty(
            self.default_seed,
            lithicrivers_core::components::Position { x: 0, y: 0, z: 0 },
        );
        self.reset_for_game();
        self.show_title();
        self.title.selected = crate::app::title_menu::WATCH_DEMO;
    }

    /// One frame of the demo: show the title card, set up a scenario, or play a beat.
    pub(crate) fn update_demo(&mut self) {
        let Some(mut demo) = self.demo.take() else {
            return;
        };
        let now = self.clock.now();
        if let Some(until) = demo.cut_until {
            if now < until {
                self.demo = Some(demo);
                return;
            }
            demo.cut_until = None;
            self.begin_scenario(&mut demo, now);
        }
        let finished = demo.cursor >= demo.scenarios[demo.index].steps.len()
            && demo.pending.is_empty()
            && now >= demo.next_at;
        if finished || now.saturating_duration_since(demo.scenario_started) >= SCENARIO_MAX {
            demo.index = (demo.index + 1) % demo.scenarios.len();
            demo.cut_until = Some(now + CUT);
        } else if now >= demo.next_at {
            self.play_beat(&mut demo, now);
        }
        self.demo = Some(demo);
    }

    /// Reset per-scenario state and run the scenario's setup (every step before its first
    /// `beat`) at once.
    fn begin_scenario(&mut self, demo: &mut DemoState, now: Instant) {
        demo.cursor = 0;
        demo.pending.clear();
        demo.beat = playback(DEFAULT_BEAT_MS);
        demo.scenario_started = now;
        demo.next_at = now;
        let steps = demo.scenarios[demo.index].steps.clone();
        while let Some(step) = steps.get(demo.cursor) {
            demo.cursor += 1;
            if let Step::Beat(ms) = step {
                demo.beat = playback(*ms);
                break;
            }
            self.run_demo_step(demo, step, now);
            while let Some(code) = demo.pending.pop_front() {
                self.press_demo_key(code);
            }
        }
    }

    /// Press the next queued key, or run steps until one takes a beat.
    fn play_beat(&mut self, demo: &mut DemoState, now: Instant) {
        demo.next_at = now + demo.beat;
        if let Some(code) = demo.pending.pop_front() {
            self.press_demo_key(code);
            return;
        }
        let steps = demo.scenarios[demo.index].steps.clone();
        while let Some(step) = steps.get(demo.cursor) {
            demo.cursor += 1;
            if self.run_demo_step(demo, step, now) {
                return;
            }
        }
    }

    /// Run one scenario step. Returns whether it used up the beat.
    fn run_demo_step(&mut self, demo: &mut DemoState, step: &Step, now: Instant) -> bool {
        match step {
            Step::Beat(ms) => {
                demo.beat = playback(*ms);
                false
            }
            Step::SkipSplash => {
                self.start_default_world();
                false
            }
            Step::World(t) => {
                self.start_default_world_of_type(*t);
                false
            }
            Step::Key {
                category,
                action,
                count,
            } => {
                let code = self
                    .ui
                    .keybinds
                    .get_map()
                    .get(&format!("{category}:{action}"))
                    .and_then(|codes| codes.first().copied());
                if let Some(code) = code {
                    demo.pending
                        .extend(std::iter::repeat_n(code, *count as usize));
                    if let Some(first) = demo.pending.pop_front() {
                        self.press_demo_key(first);
                    }
                }
                true
            }
            Step::RawKey { code, .. } => {
                self.press_demo_key(*code);
                true
            }
            Step::Type(text) => {
                demo.pending.extend(text.chars().map(KeyCode::Char));
                if let Some(first) = demo.pending.pop_front() {
                    self.press_demo_key(first);
                }
                true
            }
            Step::Wait(ms) => {
                demo.next_at = now + playback(*ms);
                true
            }
            Step::Tick(n) => {
                for _ in 0..*n {
                    let result = self.core.game.tick();
                    self.apply_tick_result(&result);
                }
                true
            }
            Step::Mouse(input) => {
                crate::app::input::mouse::dispatch_mouse(self, *input);
                true
            }
            Step::Cheat(command) => {
                let _ =
                    crate::app::input::cheat_console::get_command_registry().execute(self, command);
                true
            }
            Step::Screen(_)
            | Step::State(_)
            | Step::ExpectText(_)
            | Step::SaveReload
            | Step::Quit => false,
        }
    }

    fn press_demo_key(&mut self, code: KeyCode) {
        let _ = crate::app::input_handler::dispatch(self, code);
        // A scripted key never leaves the main UI for a title screen
        if self.splash.state != SplashState::MainUI {
            self.splash.state = SplashState::MainUI;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::{chain_commands, run_script, Overrides, Runner, EXIT_OK};

    /// From a fresh client: through the logo screens to the title, then Watch Demo.
    const START_DEMO: &[&str] = &[
        "rawkey ENTER",
        "rawkey ENTER",
        "key title.DOWN",
        "key title.SELECT",
    ];

    /// Each scenario runs as an ordinary script (`beat` is a no-op there) and its last
    /// frame is a snapshot, so a scenario broken by a game change fails here.
    #[test]
    fn every_scenario_runs_as_a_script() {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("../snapshots");
        for file in SCENARIOS {
            let text = format!("seed 12345\nsize 100x30\n{}", scenario_text(file));
            let (mut out, mut err_out) = (Vec::new(), Vec::new());
            let code = run_script(&text, 0, Overrides::default(), &mut out, &mut err_out);
            assert_eq!(
                code,
                EXIT_OK,
                "demo scenario {file} failed:\n{}",
                String::from_utf8_lossy(&err_out)
            );
            settings.bind(|| {
                insta::assert_snapshot!(format!("demo_{file}"), String::from_utf8(out).unwrap())
            });
        }
        assert_eq!(scenarios().len(), SCENARIOS.len());
    }

    #[test]
    fn watch_demo_shows_a_title_card_then_the_banner() {
        let mut runner = chain_commands(START_DEMO);
        assert!(runner.app.demo.is_some());
        let card = runner.screen_text();
        assert!(card.contains("WAKING UP IN THE BUNKER"), "{card}");
        runner.chain(&["wait 2000"]);
        let screen = runner.screen_text();
        let top = screen.lines().next().unwrap();
        assert!(top.contains("DEMO   DEMO"), "{top}");
        assert!(top.contains("WAKING UP IN THE BUNKER"));
        assert!(
            screen.contains("MODES"),
            "the real game UI plays underneath"
        );
    }

    #[test]
    fn only_escape_leaves_the_demo_and_nothing_is_saved() {
        let mut runner = chain_commands(START_DEMO);
        runner.chain(&[
            "wait 2000",
            "key movement.MOVE_NORTH 3",
            "key title.SELECT",
            "rawkey q",
        ]);
        assert!(runner.app.demo.is_some(), "other keys are ignored");
        runner.chain(&["key title.BACK"]);
        assert!(runner.app.demo.is_none());
        assert_eq!(runner.app.splash.state, crate::SplashState::Title);
        assert_eq!(
            runner.app.title.selected,
            crate::app::title_menu::WATCH_DEMO
        );
        let saved = std::fs::read_dir(&runner.app.saves_dir).unwrap().count();
        assert_eq!(saved, 0, "demo worlds are never saved");
    }

    #[test]
    fn scenarios_cut_in_order_and_loop() {
        let mut runner = chain_commands(START_DEMO);
        let mut seen = vec![0];
        // At most 20 s plus the card per scenario; sample every half second
        let total = SCENARIOS.len() as u64 * (SCENARIO_MAX + CUT).as_millis() as u64 + 2000;
        for _ in 0..total / 500 {
            runner.chain(&["wait 500"]);
            let index = runner.app.demo.as_ref().unwrap().index;
            if seen.last() != Some(&index) {
                seen.push(index);
            }
        }
        let expected: Vec<usize> = (0..SCENARIOS.len()).chain([0]).collect();
        assert_eq!(&seen[..expected.len().min(seen.len())], &expected[..]);
    }

    #[test]
    fn the_demo_is_the_same_every_time() {
        let run = || {
            let mut runner = Runner::new(12345, (100, 30));
            runner.chain(START_DEMO);
            let mut frames = String::new();
            for _ in 0..12 {
                runner.chain(&["wait 1500"]);
                frames.push_str(&runner.screen_text());
            }
            frames
        };
        assert_eq!(run(), run());
    }
}
