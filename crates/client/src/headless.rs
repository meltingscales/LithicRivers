//! Scripted, terminal-free runs of the full client (`just client-blind <script>`).
//!
//! A script is one step per line; `#` starts a comment. See AGENTS.md for the command list.
//! Runs use a fake clock, so the same script, seed and build always print the same output.

use crate::app::handle_input;
use crate::app::input::cheat_console::get_command_registry;
use crate::app_state::{BodyRepairState, CheatConsoleState, TutorialSelectionState};
use crate::{step, ui, App};
use crossterm::event::KeyCode;
use lithicrivers_core::components::{
    Combat, DroppedItem, EntityKind, Health, Inventory, Player, Position,
};
use lithicrivers_core::model::body::Body;
use ratatui::{backend::TestBackend, Terminal};
use serde::Serialize;
use std::io::Write;
use std::time::Duration;
use unicode_width::UnicodeWidthStr;

pub const DEFAULT_SIZE: (u16, u16) = (160, 50);
/// Length of one simulated frame; matches the interactive loop's input poll interval.
const FRAME: Duration = Duration::from_millis(50);
const NEARBY_RADIUS: i64 = 12;
const MAX_NEARBY: usize = 32;
const MAX_MESSAGES: usize = 20;

pub const EXIT_OK: i32 = 0;
pub const EXIT_SCRIPT_ERROR: i32 = 1;
pub const EXIT_EXPECT_FAILED: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    SkipSplash,
    Key {
        category: String,
        action: String,
        count: u32,
    },
    RawKey {
        name: String,
        code: KeyCode,
    },
    Type(String),
    Wait(u64),
    Tick(u32),
    Cheat(String),
    Screen(Option<String>),
    State(Option<String>),
    ExpectText(String),
    SaveReload,
    Quit,
}

/// Renders a step back to script syntax; parsing the result gives the same step.
impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::SkipSplash => write!(f, "skip-splash"),
            Step::Key {
                category,
                action,
                count: 1,
            } => write!(f, "key {category}.{action}"),
            Step::Key {
                category,
                action,
                count,
            } => write!(f, "key {category}.{action} {count}"),
            Step::RawKey { name, .. } => write!(f, "rawkey {name}"),
            Step::Type(text) => write!(f, "type \"{text}\""),
            Step::Wait(ms) => write!(f, "wait {ms}"),
            Step::Tick(n) => write!(f, "tick {n}"),
            Step::Cheat(command) => write!(f, "cheat {command}"),
            Step::Screen(None) => write!(f, "screen"),
            Step::Screen(Some(label)) => write!(f, "screen {label}"),
            Step::State(None) => write!(f, "state"),
            Step::State(Some(label)) => write!(f, "state {label}"),
            Step::ExpectText(text) => write!(f, "expect-text \"{text}\""),
            Step::SaveReload => write!(f, "save-reload"),
            Step::Quit => write!(f, "quit"),
        }
    }
}

impl Step {
    /// Whether running this step can change game or UI state (as opposed to only reading it).
    pub fn changes_state(&self) -> bool {
        !matches!(
            self,
            Step::Screen(_) | Step::State(_) | Step::ExpectText(_) | Step::Quit
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptLine {
    pub line: usize,
    pub step: Step,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Script {
    pub seed: Option<u64>,
    pub size: Option<(u16, u16)>,
    pub steps: Vec<ScriptLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptError {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for ScriptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

fn err(line: usize, message: impl Into<String>) -> ScriptError {
    ScriptError {
        line,
        message: message.into(),
    }
}

/// Parse a `"quoted string"` argument.
fn parse_quoted(line: usize, cmd: &str, rest: &str) -> Result<String, ScriptError> {
    let rest = rest.trim();
    if rest.len() >= 2 && rest.starts_with('"') && rest.ends_with('"') {
        Ok(rest[1..rest.len() - 1].to_string())
    } else {
        Err(err(line, format!("{cmd} expects a \"quoted string\"")))
    }
}

fn parse_num<T: std::str::FromStr>(line: usize, cmd: &str, arg: &str) -> Result<T, ScriptError> {
    arg.parse()
        .map_err(|_| err(line, format!("{cmd}: invalid number '{arg}'")))
}

fn optional_label(rest: &str) -> Option<String> {
    let rest = rest.trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

/// Parse script text. Checks syntax and setup ordering only; see `validate` for
/// checks that need a running app (keybind and cheat names).
pub fn parse_script(text: &str) -> Result<Script, ScriptError> {
    let mut script = Script {
        seed: None,
        size: None,
        steps: Vec::new(),
    };

    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (cmd, rest) = trimmed
            .split_once(char::is_whitespace)
            .map_or((trimmed, ""), |(c, r)| (c, r.trim()));
        let args: Vec<&str> = rest.split_whitespace().collect();
        let expect_args = |n: usize| -> Result<(), ScriptError> {
            if args.len() == n {
                Ok(())
            } else {
                Err(err(line, format!("{cmd} expects {n} argument(s)")))
            }
        };

        let step = match cmd {
            "seed" | "size" => {
                if !script.steps.is_empty() {
                    return Err(err(line, format!("{cmd} must come before other steps")));
                }
                expect_args(1)?;
                if cmd == "seed" {
                    script.seed = Some(parse_num(line, cmd, args[0])?);
                } else {
                    script.size = Some(
                        parse_size(args[0])
                            .ok_or_else(|| err(line, "size expects <cols>x<rows>"))?,
                    );
                }
                continue;
            }
            "skip-splash" => {
                expect_args(0)?;
                Step::SkipSplash
            }
            "key" => {
                if args.is_empty() || args.len() > 2 {
                    return Err(err(line, "key expects <category>.<ACTION> [count]"));
                }
                let (category, action) = args[0]
                    .split_once('.')
                    .ok_or_else(|| err(line, "key expects <category>.<ACTION>"))?;
                let count = match args.get(1) {
                    Some(n) => parse_num(line, cmd, n)?,
                    None => 1,
                };
                Step::Key {
                    category: category.to_string(),
                    action: action.to_string(),
                    count,
                }
            }
            "rawkey" => {
                expect_args(1)?;
                let code = lithicrivers_core::keycode_mapping::try_parse_keycode(args[0])
                    .ok_or_else(|| err(line, format!("rawkey: unknown key name '{}'", args[0])))?;
                Step::RawKey {
                    name: args[0].to_string(),
                    code,
                }
            }
            "type" => Step::Type(parse_quoted(line, cmd, rest)?),
            "wait" => {
                expect_args(1)?;
                Step::Wait(parse_num(line, cmd, args[0])?)
            }
            "tick" => match args.len() {
                0 => Step::Tick(1),
                1 => Step::Tick(parse_num(line, cmd, args[0])?),
                _ => return Err(err(line, "tick expects [n]")),
            },
            "cheat" => {
                if rest.is_empty() {
                    return Err(err(line, "cheat expects a command line"));
                }
                Step::Cheat(rest.to_string())
            }
            "screen" => Step::Screen(optional_label(rest)),
            "state" => Step::State(optional_label(rest)),
            "expect-text" => Step::ExpectText(parse_quoted(line, cmd, rest)?),
            "save-reload" => {
                expect_args(0)?;
                Step::SaveReload
            }
            "quit" => {
                expect_args(0)?;
                Step::Quit
            }
            other => return Err(err(line, format!("unknown command '{other}'"))),
        };
        script.steps.push(ScriptLine { line, step });
    }
    Ok(script)
}

/// Check keybind and cheat names against the app's loaded config and command registry.
pub fn validate(script: &Script, app: &App) -> Result<(), ScriptError> {
    validate_steps(&script.steps, app)
}

pub fn validate_steps(steps: &[ScriptLine], app: &App) -> Result<(), ScriptError> {
    let registry = get_command_registry();
    for ScriptLine { line, step } in steps {
        match step {
            Step::Key {
                category, action, ..
            } => {
                let bound = app
                    .ui
                    .keybinds
                    .get_map()
                    .get(&format!("{category}:{action}"))
                    .is_some_and(|codes| !codes.is_empty());
                if !bound {
                    return Err(err(
                        *line,
                        format!("unknown or unbound keybind '{category}.{action}'"),
                    ));
                }
            }
            Step::Cheat(command) => {
                let name = command.split_whitespace().next().unwrap_or_default();
                if !registry.get_matching_commands(name).contains(&name) {
                    return Err(err(*line, format!("unknown cheat command '{name}'")));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Why a step stopped the run.
#[derive(Debug)]
pub enum StepFailure {
    Error(ScriptError),
    Expectation {
        line: usize,
        expected: String,
        screen: String,
    },
}

impl StepFailure {
    /// Exit status and the message a script run prints for this failure.
    fn report(&self) -> (i32, String) {
        match self {
            StepFailure::Error(e) => (EXIT_SCRIPT_ERROR, format!("script error: {e}\n")),
            StepFailure::Expectation {
                line,
                expected,
                screen,
            } => (
                EXIT_EXPECT_FAILED,
                format!(
                    "expectation failed at line {line}: screen does not contain \"{expected}\"\n\
                     === screen ===\n{screen}=== end screen ===\n"
                ),
            ),
        }
    }
}

/// Outcome of running steps against a session.
#[derive(Debug)]
pub struct StepsReport {
    /// `EXIT_OK`, `EXIT_SCRIPT_ERROR` or `EXIT_EXPECT_FAILED`
    pub status: i32,
    /// Screen and state dumps, in order
    pub output: String,
    /// What went wrong, if a step failed
    pub error: Option<String>,
    /// Steps that ran successfully, in order (excludes the failing step)
    pub completed: Vec<ScriptLine>,
}

/// A headless client session: the app, an in-memory terminal, and the dump output.
pub struct Runner {
    pub app: App,
    terminal: Terminal<TestBackend>,
    _saves: TempSaves,
}

/// A new, empty saves folder for one run, removed when the run ends.
struct TempSaves(std::path::PathBuf);

impl TempSaves {
    fn new() -> Self {
        static RUNS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = RUNS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("lithicrivers-headless-{}-{n}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create headless saves folder");
        Self(dir)
    }
}

impl Drop for TempSaves {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Runner {
    pub fn new(seed: u64, size: (u16, u16)) -> Self {
        let saves = TempSaves::new();
        let mut app = App::new_headless(seed);
        app.saves_dir = saves.0.clone();
        let terminal =
            Terminal::new(TestBackend::new(size.0, size.1)).expect("TestBackend never fails");
        let mut runner = Self {
            app,
            terminal,
            _saves: saves,
        };
        runner.frame();
        runner
    }

    /// One iteration of the interactive loop without input: update, then draw.
    fn frame(&mut self) {
        step(&mut self.app);
        self.draw();
    }

    fn draw(&mut self) {
        let app = &mut self.app;
        self.terminal
            .draw(|f| ui(f, app))
            .expect("TestBackend never fails");
    }

    /// Feed one key press the way the interactive loop does, then run a frame.
    fn press(&mut self, line: usize, code: KeyCode) -> Result<(), StepFailure> {
        handle_input(&mut self.app, code)
            .map_err(|e| StepFailure::Error(err(line, format!("input error: {e}"))))?;
        self.frame();
        Ok(())
    }

    /// The current screen as plain text, one line per row, trailing spaces trimmed.
    pub fn screen_text(&mut self) -> String {
        self.draw();
        screen_text(self.terminal.backend())
    }

    /// Run steps in order, stopping at the first failure or `quit`. Never panics on a bad
    /// step: failures come back in the report.
    pub fn run_steps(&mut self, steps: &[ScriptLine]) -> StepsReport {
        let mut report = StepsReport {
            status: EXIT_OK,
            output: String::new(),
            error: None,
            completed: Vec::new(),
        };
        for script_line in steps {
            match self.exec(script_line, &mut report.output) {
                Ok(keep_going) => {
                    report.completed.push(script_line.clone());
                    if !keep_going {
                        break;
                    }
                }
                Err(failure) => {
                    let (status, message) = failure.report();
                    report.status = status;
                    report.error = Some(message);
                    break;
                }
            }
        }
        report
    }

    /// Execute one step, appending any dump to `out`. Returns `Ok(false)` on `quit`.
    pub fn exec(
        &mut self,
        script_line: &ScriptLine,
        out: &mut String,
    ) -> Result<bool, StepFailure> {
        let line = script_line.line;
        match &script_line.step {
            Step::SkipSplash => {
                self.app.start_default_world();
                self.frame();
            }
            Step::Key {
                category,
                action,
                count,
            } => {
                let code = self
                    .app
                    .ui
                    .keybinds
                    .get_map()
                    .get(&format!("{category}:{action}"))
                    .and_then(|codes| codes.first().copied())
                    .ok_or_else(|| {
                        StepFailure::Error(err(
                            line,
                            format!("unbound keybind '{category}.{action}'"),
                        ))
                    })?;
                for _ in 0..*count {
                    self.press(line, code)?;
                }
            }
            Step::RawKey { code, .. } => self.press(line, *code)?,
            Step::Type(text) => {
                for c in text.chars() {
                    self.press(line, KeyCode::Char(c))?;
                }
            }
            Step::Wait(ms) => {
                let mut remaining = Duration::from_millis(*ms);
                while !remaining.is_zero() {
                    let dt = remaining.min(FRAME);
                    self.app.clock.advance(dt);
                    remaining -= dt;
                    self.frame();
                }
            }
            Step::Tick(n) => {
                for _ in 0..*n {
                    let result = self.app.core.game.tick();
                    self.app.apply_tick_result(&result);
                }
                self.frame();
            }
            Step::Cheat(command) => {
                get_command_registry()
                    .execute(&mut self.app, command)
                    .map_err(|e| StepFailure::Error(err(line, format!("cheat failed: {e}"))))?;
                self.frame();
            }
            Step::Screen(label) => {
                let label = label.clone().unwrap_or_else(|| line.to_string());
                let text = self.screen_text();
                out.push_str(&format!(
                    "=== screen {label} ===\n{text}=== end screen ===\n"
                ));
            }
            Step::State(label) => {
                let label = label.clone().unwrap_or_else(|| line.to_string());
                let json = state_json(&self.app);
                out.push_str(&format!(
                    "=== state {label} ===\n{json}\n=== end state ===\n"
                ));
            }
            Step::ExpectText(expected) => {
                let screen = self.screen_text();
                if !screen.contains(expected.as_str()) {
                    return Err(StepFailure::Expectation {
                        line,
                        expected: expected.clone(),
                        screen,
                    });
                }
            }
            Step::SaveReload => {
                let saved = self.app.save_to_string().map_err(|e| {
                    StepFailure::Error(err(line, format!("save-reload: save failed: {e:#}")))
                })?;
                self.app.load_from_str(&saved).map_err(|e| {
                    StepFailure::Error(err(line, format!("save-reload: load failed: {e:#}")))
                })?;
            }
            Step::Quit => return Ok(false),
        }
        Ok(true)
    }
}

/// Render a TestBackend buffer as text, skipping cells hidden by wide characters
/// (same rule as ratatui's own buffer view).
fn screen_text(backend: &TestBackend) -> String {
    let buffer = backend.buffer();
    let mut text = String::new();
    for row in buffer.content.chunks(buffer.area.width as usize) {
        let mut line = String::new();
        let mut skip = 0usize;
        for cell in row {
            if skip == 0 {
                line.push_str(cell.symbol());
            }
            skip = skip.max(cell.symbol().width()).saturating_sub(1);
        }
        text.push_str(line.trim_end());
        text.push('\n');
    }
    text
}

/// Command-line overrides for a run. These win over the script's own `seed`/`size`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Overrides {
    pub seed: Option<u64>,
    pub size: Option<(u16, u16)>,
}

/// Parse `<cols>x<rows>`.
pub fn parse_size(s: &str) -> Option<(u16, u16)> {
    let (cols, rows) = s.split_once('x')?;
    Some((cols.parse().ok()?, rows.parse().ok()?))
}

/// Run a whole script, writing dumps to `out` and errors to `err_out`. Returns the exit status.
pub fn run_script(
    text: &str,
    default_seed: u64,
    overrides: Overrides,
    out: &mut impl Write,
    err_out: &mut impl Write,
) -> i32 {
    let script = match parse_script(text) {
        Ok(s) => s,
        Err(e) => {
            let _ = writeln!(err_out, "script error: {e}");
            return EXIT_SCRIPT_ERROR;
        }
    };
    let mut runner = Runner::new(
        overrides.seed.or(script.seed).unwrap_or(default_seed),
        overrides.size.or(script.size).unwrap_or(DEFAULT_SIZE),
    );
    if let Err(e) = validate(&script, &runner.app) {
        let _ = writeln!(err_out, "script error: {e}");
        return EXIT_SCRIPT_ERROR;
    }

    let report = runner.run_steps(&script.steps);
    let _ = out.write_all(report.output.as_bytes());
    if let Some(error) = &report.error {
        let _ = err_out.write_all(error.as_bytes());
    }
    let _ = out.flush();
    report.status
}

// ---------------------------------------------------------------------------
// State snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AgentState {
    pub version: u32,
    pub seed: u64,
    pub tick: u64,
    pub screen: String,
    pub tab: String,
    pub modal: Option<&'static str>,
    pub player: Option<PlayerSnapshot>,
    pub combat: CombatSnapshot,
    pub nearby: Vec<EntitySnapshot>,
    pub messages: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PlayerSnapshot {
    pub position: [i64; 3],
    pub body: Vec<BodyPartSnapshot>,
    pub inventory: Vec<ItemSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct BodyPartSnapshot {
    pub part: String,
    pub state: String,
    pub integrity: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemSnapshot {
    pub item: String,
    pub qty: u32,
}

#[derive(Debug, Serialize)]
pub struct CombatSnapshot {
    pub active: bool,
    pub enemies: Vec<EntitySnapshot>,
}

#[derive(Debug, Serialize)]
pub struct EntitySnapshot {
    pub kind: String,
    pub position: [i64; 3],
    pub distance: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub qty: Option<u32>,
}

fn active_modal(app: &App) -> Option<&'static str> {
    let p = &app.panels;
    if matches!(p.cheat_console, CheatConsoleState::Open { .. }) {
        Some("cheat_console")
    } else if matches!(
        p.tutorial_selection,
        TutorialSelectionState::SelectingTutorial { .. }
    ) {
        Some("tutorial_selection")
    } else if matches!(
        p.body_repair,
        BodyRepairState::SelectingRepairAndPart { .. }
    ) {
        Some("body_repair")
    } else if p.multi_action_select != Default::default() {
        Some("multi_action_select")
    } else if p.hotbar_assignment != Default::default() {
        Some("hotbar_assignment")
    } else if p.npc_interaction != Default::default() {
        Some("npc_interaction")
    } else if p.corpse_looting != Default::default() {
        Some("corpse_looting")
    } else {
        None
    }
}

fn entity_sort_key(e: &EntitySnapshot) -> (i64, [i64; 3], String) {
    (e.distance, e.position, e.kind.clone())
}

/// Build the agent-facing state snapshot. Lists are sorted and capped; no tile data.
pub fn agent_state(app: &App) -> AgentState {
    let game = &app.core.game;
    let world = &game.world;

    let player = world
        .query::<(&Player, &Position, &Body, &Inventory)>()
        .iter()
        .next()
        .map(|(_, (_, pos, body, inv))| {
            let mut body: Vec<BodyPartSnapshot> = body
                .parts
                .values()
                .map(|p| BodyPartSnapshot {
                    part: format!("{:?}", p.part_type),
                    state: format!("{:?}", p.state),
                    integrity: p.integrity,
                })
                .collect();
            body.sort_by(|a, b| a.part.cmp(&b.part));
            let mut inventory: Vec<ItemSnapshot> = inv
                .slots
                .iter()
                .map(|s| ItemSnapshot {
                    item: format!("{:?}", s.kind),
                    qty: s.qty,
                })
                .collect();
            inventory.sort_by(|a, b| a.item.cmp(&b.item).then(a.qty.cmp(&b.qty)));
            PlayerSnapshot {
                position: [pos.x, pos.y, pos.z],
                body,
                inventory,
            }
        });

    let origin = player.as_ref().map_or([0, 0, 0], |p| p.position);
    let distance = |pos: &Position| {
        (pos.x - origin[0])
            .abs()
            .max((pos.y - origin[1]).abs())
            .max((pos.z - origin[2]).abs())
    };

    let mut nearby: Vec<EntitySnapshot> = Vec::new();
    let mut enemies: Vec<EntitySnapshot> = Vec::new();
    for (_, (pos, kind, health, combat)) in world
        .query::<(&Position, &EntityKind, Option<&Health>, Option<&Combat>)>()
        .iter()
    {
        if *kind == EntityKind::Player {
            continue;
        }
        let snapshot = || EntitySnapshot {
            kind: format!("{kind:?}"),
            position: [pos.x, pos.y, pos.z],
            distance: distance(pos),
            health: health.map(|h| h.current),
            qty: None,
        };
        if combat.is_some_and(|c| c.triggered) {
            enemies.push(snapshot());
        }
        if distance(pos) <= NEARBY_RADIUS {
            nearby.push(snapshot());
        }
    }
    for (_, (pos, item)) in world.query::<(&Position, &DroppedItem)>().iter() {
        if distance(pos) <= NEARBY_RADIUS {
            nearby.push(EntitySnapshot {
                kind: format!("Item:{:?}", item.kind),
                position: [pos.x, pos.y, pos.z],
                distance: distance(pos),
                health: None,
                qty: Some(item.qty),
            });
        }
    }
    nearby.sort_by_key(entity_sort_key);
    nearby.truncate(MAX_NEARBY);
    enemies.sort_by_key(entity_sort_key);

    let log = &game.res.events.get_message_log().messages;
    let messages = log
        .iter()
        .skip(log.len().saturating_sub(MAX_MESSAGES))
        .map(|m| m.text.clone())
        .collect();

    AgentState {
        version: 1,
        seed: game.res.world_state.seed,
        tick: game.res.time.tick,
        screen: format!("{:?}", app.splash.state),
        tab: format!("{:?}", app.ui.current_tab),
        modal: active_modal(app),
        player,
        combat: CombatSnapshot {
            active: app.combat.is_active(),
            enemies,
        },
        nearby,
        messages,
    }
}

pub fn state_json(app: &App) -> String {
    serde_json::to_string_pretty(&agent_state(app)).expect("snapshot serializes")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: u64 = 12345;

    fn run(text: &str) -> (i32, String, String) {
        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        let code = run_script(text, SEED, Overrides::default(), &mut out, &mut err_out);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err_out).unwrap(),
        )
    }

    fn parse_err(text: &str) -> ScriptError {
        parse_script(text).expect_err("script should not parse")
    }

    fn player_pos(app: &App) -> Position {
        app.core.game.get_player_position().unwrap()
    }

    // --- 1.3 / 2.2: frame step under a fake clock -------------------------------

    #[test]
    fn headless_app_steps_without_audio_or_terminal() {
        let mut runner = Runner::new(SEED, (80, 24));
        runner.frame();
        assert!(matches!(runner.app.clock, crate::clock::Clock::Fake { .. }));
    }

    #[test]
    fn combat_auto_ticks_follow_fake_time() {
        let mut runner = Runner::new(SEED, (80, 24));
        runner
            .exec(
                &ScriptLine {
                    line: 1,
                    step: Step::SkipSplash,
                },
                &mut String::new(),
            )
            .unwrap();
        runner.app.combat = crate::app_state::CombatUiState::Active {
            current_move: 0,
            current_enemy: 0,
            enemy_timers: Vec::new(),
            move_scroll_offset: 0,
        };
        runner.frame(); // the frame after combat starts arms the tick timer
        let before = runner.app.core.game.res.time.tick;

        runner
            .exec(
                &ScriptLine {
                    line: 2,
                    step: Step::Wait(3000),
                },
                &mut String::new(),
            )
            .unwrap();

        // COMBAT_MS_PER_TICK defaults to 1000 and an idle tick costs 1
        assert_eq!(runner.app.core.game.res.time.tick - before, 3);
    }

    // --- 3.1: parsing ------------------------------------------------------------

    #[test]
    fn parses_every_command() {
        let script = parse_script(
            "# comment\n\nseed 7\nsize 80x24\nskip-splash\nkey movement.MOVE_NORTH 3\n\
             rawkey ENTER\ntype \"hi there\"\nwait 250\ntick\ntick 4\ncheat tp 1 2 3\n\
             screen\nscreen after\nstate\nexpect-text \"Messages\"\nsave-reload\nquit\n",
        )
        .unwrap();
        assert_eq!(script.seed, Some(7));
        assert_eq!(script.size, Some((80, 24)));
        let steps: Vec<Step> = script.steps.into_iter().map(|l| l.step).collect();
        assert_eq!(
            steps,
            vec![
                Step::SkipSplash,
                Step::Key {
                    category: "movement".into(),
                    action: "MOVE_NORTH".into(),
                    count: 3
                },
                Step::RawKey {
                    name: "ENTER".into(),
                    code: KeyCode::Enter
                },
                Step::Type("hi there".into()),
                Step::Wait(250),
                Step::Tick(1),
                Step::Tick(4),
                Step::Cheat("tp 1 2 3".into()),
                Step::Screen(None),
                Step::Screen(Some("after".into())),
                Step::State(None),
                Step::ExpectText("Messages".into()),
                Step::SaveReload,
                Step::Quit,
            ]
        );
    }

    #[test]
    fn parse_errors_name_the_line() {
        assert_eq!(parse_err("skip-splash\njump 3").line, 2);
        assert!(parse_err("jump 3")
            .message
            .contains("unknown command 'jump'"));
        assert_eq!(parse_err("skip-splash\nseed 5").line, 2);
        assert_eq!(parse_err("size 80by24").line, 1);
        assert_eq!(parse_err("wait soon").line, 1);
        assert_eq!(parse_err("key MOVE_NORTH").line, 1);
        assert_eq!(parse_err("type hello").line, 1);
        assert_eq!(parse_err("rawkey NOT_A_KEY").line, 1);
        assert_eq!(parse_err("cheat").line, 1);
    }

    // --- 3.2: validation against config and cheat registry -------------------------

    #[test]
    fn unknown_keybind_and_cheat_fail_before_running() {
        let (code, out, err_out) = run("skip-splash\nkey movement.FLY\nscreen");
        assert_eq!(code, EXIT_SCRIPT_ERROR);
        assert!(out.is_empty(), "nothing runs when validation fails");
        assert!(err_out.contains("line 2") && err_out.contains("movement.FLY"));

        let (code, _, err_out) = run("skip-splash\n\ncheat summon_dragon");
        assert_eq!(code, EXIT_SCRIPT_ERROR);
        assert!(err_out.contains("line 3") && err_out.contains("summon_dragon"));
    }

    // --- 4.x: runner, screen dumps, expectations --------------------------------

    #[test]
    fn keybind_press_moves_the_player() {
        let mut runner = Runner::new(SEED, (80, 24));
        let script = parse_script("skip-splash\nkey movement.MOVE_SOUTH").unwrap();
        runner.exec(&script.steps[0], &mut String::new()).unwrap();
        let start = player_pos(&runner.app);
        runner.exec(&script.steps[1], &mut String::new()).unwrap();
        let end = player_pos(&runner.app);
        assert_eq!((end.x, end.y, end.z), (start.x, start.y + 1, start.z));
    }

    #[test]
    fn screen_dump_fits_the_screen_size() {
        let (code, out, _) = run("size 90x25\nskip-splash\nscreen start");
        assert_eq!(code, EXIT_OK);
        let body = out
            .strip_prefix("=== screen start ===\n")
            .and_then(|s| s.strip_suffix("=== end screen ===\n"))
            .expect("framed screen dump");
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(lines.len(), 25);
        assert!(lines.iter().all(|l| l.width() <= 90 && l == &l.trim_end()));
        assert!(body.contains("Messages"));
    }

    #[test]
    fn expectations_set_the_exit_status() {
        let (code, _, _) = run("skip-splash\nexpect-text \"Messages\"");
        assert_eq!(code, EXIT_OK);

        let (code, _, err_out) = run("skip-splash\nexpect-text \"You chop the tree\"");
        assert_eq!(code, EXIT_EXPECT_FAILED);
        assert!(err_out.contains("line 2") && err_out.contains("=== screen ==="));

        let (code, _, _) = run("skip-splash\nquit\nexpect-text \"never checked\"");
        assert_eq!(code, EXIT_OK);
    }

    // --- 5.x: state snapshot --------------------------------------------------------

    #[test]
    fn state_snapshot_reports_player_and_inventory() {
        let runner = Runner::new(SEED, (80, 24));
        let state = agent_state(&runner.app);
        assert_eq!(state.seed, SEED);
        let player = state.player.expect("player present");
        let pos = player_pos(&runner.app);
        assert_eq!(player.position, [pos.x, pos.y, pos.z]);
        let items: Vec<(&str, u32)> = player
            .inventory
            .iter()
            .map(|i| (i.item.as_str(), i.qty))
            .collect();
        for expected in [("Nail", 10), ("Log", 3), ("Torch", 3)] {
            assert!(
                items.contains(&expected),
                "missing {expected:?} in {items:?}"
            );
        }
        assert!(state.nearby.len() <= MAX_NEARBY && state.messages.len() <= MAX_MESSAGES);
    }

    #[test]
    fn state_json_is_stable_and_has_no_tiles() {
        let runner = Runner::new(SEED, (80, 24));
        let first = state_json(&runner.app);
        assert_eq!(first, state_json(&runner.app));
        // Top-level keys appear in a fixed order
        let positions: Vec<usize> = [
            "version", "seed", "tick", "screen", "tab", "modal", "player", "combat", "nearby",
            "messages",
        ]
        .iter()
        .map(|k| first.find(&format!("\n  \"{k}\":")).expect(k))
        .collect();
        assert!(positions.windows(2).all(|w| w[0] < w[1]));
        assert!(
            !first.to_lowercase().contains("tile") && !first.to_lowercase().contains("chunk\"")
        );

        let (code, out, _) = run("state snap");
        assert_eq!(code, EXIT_OK);
        assert!(
            out.starts_with("=== state snap ===\n{") && out.ends_with("}\n=== end state ===\n")
        );
    }
}

#[cfg(test)]
mod smoke_tests {
    use super::*;

    #[test]
    fn smoke_script_passes_and_repeats_byte_for_byte() {
        let script = include_str!("../../../scripts/agent/smoke.txt");
        let run_once = || {
            let (mut out, mut err_out) = (Vec::new(), Vec::new());
            let code = run_script(script, 0, Overrides::default(), &mut out, &mut err_out);
            assert_eq!(code, EXIT_OK, "{}", String::from_utf8_lossy(&err_out));
            out
        };
        let first = run_once();
        assert!(!first.is_empty());
        assert_eq!(first, run_once());
    }

    /// Bug scripts under `scripts/agent/bugs/` must keep passing once fixed.
    #[test]
    fn combat_started_from_tutorial_tab_shows_combat_panel() {
        let script = include_str!("../../../scripts/agent/bugs/combat-on-tutorial-tab.txt");
        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        let code = run_script(script, 0, Overrides::default(), &mut out, &mut err_out);
        assert_eq!(code, EXIT_OK, "{}", String::from_utf8_lossy(&err_out));
    }

    #[test]
    fn walking_away_from_combat_closes_combat_panel() {
        let script =
            include_str!("../../../scripts/agent/bugs/combat-panel-stays-after-walking-away.txt");
        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        let code = run_script(script, 0, Overrides::default(), &mut out, &mut err_out);
        assert_eq!(code, EXIT_OK, "{}", String::from_utf8_lossy(&err_out));
    }
}

#[cfg(test)]
mod render_tests {
    use super::*;

    #[test]
    fn every_step_renders_back_to_the_same_step() {
        let text = "skip-splash\nkey movement.MOVE_NORTH\nkey movement.MOVE_SOUTH 3\nrawkey ENTER\n\
                    type \"say \"hi\" now\"\nwait 250\ntick 4\ncheat tp 1 2 3\nscreen\nscreen after\n\
                    state\nstate end\nexpect-text \"Messages\"\nsave-reload\nquit\n";
        let steps: Vec<Step> = parse_script(text)
            .unwrap()
            .steps
            .into_iter()
            .map(|l| l.step)
            .collect();
        let rendered: String = steps.iter().map(|s| format!("{s}\n")).collect();
        assert_eq!(rendered, text);
        let reparsed: Vec<Step> = parse_script(&rendered)
            .unwrap()
            .steps
            .into_iter()
            .map(|l| l.step)
            .collect();
        assert_eq!(reparsed, steps);
    }
}

#[cfg(test)]
mod cheat_tests {
    use super::*;

    fn run(text: &str) -> (i32, String, String) {
        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        let code = run_script(text, 12345, Overrides::default(), &mut out, &mut err_out);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err_out).unwrap(),
        )
    }

    fn runner_after(text: &str) -> Runner {
        let script = parse_script(text).unwrap();
        let mut runner = Runner::new(12345, (100, 30));
        let report = runner.run_steps(&script.steps);
        assert_eq!(report.status, EXIT_OK, "{:?}", report.error);
        runner
    }

    fn player(runner: &Runner) -> Position {
        runner.app.core.game.get_player_position().unwrap()
    }

    fn dogs(runner: &Runner) -> Vec<Position> {
        let mut dogs: Vec<Position> = runner
            .app
            .core
            .game
            .world
            .query::<(&Position, &EntityKind)>()
            .iter()
            .filter(|(_, (_, k))| **k == EntityKind::FeralDog)
            .map(|(_, (p, _))| *p)
            .collect();
        dogs.sort_by_key(|p| (p.x, p.y, p.z));
        dogs
    }

    // --- 2.3: tp with relative coordinates and real errors ---------------------------

    #[test]
    fn tp_accepts_relative_coordinates() {
        let start = player(&runner_after("skip-splash"));
        let runner = runner_after("skip-splash\ncheat tp ~5 ~ ~");
        assert_eq!(
            player(&runner),
            Position {
                x: start.x + 5,
                ..start
            }
        );
    }

    #[test]
    fn bad_tp_fails_the_script() {
        let (code, _, err_out) = run("skip-splash\ncheat tp ~x 0 0");
        assert_eq!(code, EXIT_SCRIPT_ERROR);
        assert!(
            err_out.contains("line 2") && err_out.contains("~x"),
            "{err_out}"
        );
    }

    // --- 2.4: spawn_enemy ---------------------------------------------------------------

    #[test]
    fn spawn_enemy_defaults_to_two_tiles_east_then_the_dog_closes_in() {
        let mut runner = runner_after("skip-splash\ncheat tp 0 0 0\ncheat spawn_enemy feral_dog");
        assert!(dogs(&runner).contains(&Position { x: 2, y: 0, z: 0 }));
        assert!(!runner.app.core.game.res.player_state.combat_active);

        // One turn later the dog is adjacent and combat starts, panel shown
        let steps = parse_script("key movement.WAIT\nexpect-text \"Action Queue\"").unwrap();
        let report = runner.run_steps(&steps.steps);
        assert_eq!(report.status, EXIT_OK, "{:?}", report.error);
        assert!(dogs(&runner).contains(&Position { x: 1, y: 0, z: 0 }));
        assert!(runner.app.combat.is_active());
    }

    /// A `tick` step that starts combat must open the combat panel, like a player action.
    #[test]
    fn tick_step_that_starts_combat_shows_the_combat_panel() {
        let mut runner =
            runner_after("skip-splash\ncheat tp 0 0 0\ncheat spawn_enemy feral_dog\ntick");
        assert!(runner.app.core.game.res.player_state.combat_active);
        assert!(runner.app.combat.is_active());
        assert_eq!(runner.app.ui.current_tab, crate::MenuTab::World);
        assert!(runner.screen_text().contains("Action Queue"));
        let state = state_json(&runner.app);
        assert!(state.contains("\"active\": true"), "{state}");
    }

    #[test]
    fn spawn_enemy_accepts_relative_coordinates() {
        let runner =
            runner_after("skip-splash\ncheat tp 0 0 0\ncheat spawn_enemy feral_dog ~-2 ~1 ~");
        assert!(dogs(&runner).contains(&Position { x: -2, y: 1, z: 0 }));
    }

    #[test]
    fn spawn_enemy_failures_fail_the_script() {
        let (code, _, err_out) = run("skip-splash\ncheat spawn_enemy dragon");
        assert_eq!(code, EXIT_SCRIPT_ERROR);
        assert!(
            err_out.contains("dragon") && err_out.contains("feral_dog"),
            "{err_out}"
        );

        let (code, _, err_out) = run("skip-splash\ncheat spawn_enemy feral_dog ~ ~ ~");
        assert_eq!(code, EXIT_SCRIPT_ERROR);
        assert!(err_out.contains("occupied"), "{err_out}");
    }

    // --- 2.1: the in-game console reports failures and keeps running ------------------

    #[test]
    fn console_failure_is_logged_in_red_and_the_game_keeps_running() {
        let mut runner = runner_after(
            "skip-splash\nkey ui.OPEN_COMMAND_MENU\ntype \"spawn_enemy dragon\"\nrawkey ENTER",
        );
        let last = runner
            .app
            .core
            .game
            .res
            .events
            .get_message_log()
            .messages
            .last()
            .cloned()
            .unwrap();
        assert!(
            last.text.contains("Cheat failed") && last.text.contains("dragon"),
            "{}",
            last.text
        );
        assert_eq!(
            last.color,
            lithicrivers_core::message_log::MessageColor::Red
        );
        assert!(!runner.app.core.should_quit);
        // Still playable afterwards
        let report = runner.run_steps(&parse_script("key movement.MOVE_SOUTH").unwrap().steps);
        assert_eq!(report.status, EXIT_OK);
    }
}
