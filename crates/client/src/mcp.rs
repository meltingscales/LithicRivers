//! Local MCP server (`just client-mcp`): lets an agent play a headless game session
//! through tools over stdio, and export the session as a headless script.
//!
//! Every tool is one headless script step (or a fragment of them) run through the same
//! parser and executor as `just client-blind`, so tools and scripts can't diverge.
//! Stdout carries only MCP protocol messages; logs go to the log file.

use crate::headless::{
    parse_script, validate_steps, Runner, Step, DEFAULT_SIZE, EXIT_EXPECT_FAILED, EXIT_OK,
    EXIT_SCRIPT_ERROR,
};
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig};
use rmcp::{tool, tool_handler, tool_router, ErrorData, ServerHandler, ServiceExt};
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::mpsc;
use tokio::sync::oneshot;

// ---------------------------------------------------------------------------
// Session: owned by a dedicated thread (the headless App holds audio types that
// can't cross threads), driven by messages from the async tool handlers.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Setup {
    pub seed: u64,
    pub size: (u16, u16),
    pub skip_splash: bool,
}

/// What one request to the session produced.
#[derive(Debug, Clone, PartialEq)]
pub struct Reply {
    pub status: i32,
    pub output: String,
    pub error: Option<String>,
}

impl Reply {
    fn ok(output: String) -> Self {
        Self {
            status: EXIT_OK,
            output,
            error: None,
        }
    }

    fn script_error(message: impl Into<String>) -> Self {
        Self {
            status: EXIT_SCRIPT_ERROR,
            output: String::new(),
            error: Some(message.into()),
        }
    }
}

/// One game session plus the transcript of everything that changed it.
pub struct Session {
    setup: Setup,
    runner: Runner,
    transcript: Vec<Step>,
}

impl Session {
    pub fn new(setup: Setup) -> Self {
        let mut runner = Runner::new(setup.seed, setup.size);
        let mut transcript = Vec::new();
        if setup.skip_splash {
            let step = crate::headless::ScriptLine {
                line: 0,
                step: Step::SkipSplash,
            };
            let report = runner.run_steps(std::slice::from_ref(&step));
            debug_assert_eq!(report.status, EXIT_OK);
            transcript.push(Step::SkipSplash);
        }
        Self {
            setup,
            runner,
            transcript,
        }
    }

    /// Run script text against this session. `seed`/`size` lines are rejected: start a
    /// new game to change those. State-changing steps that succeed are recorded.
    pub fn run(&mut self, text: &str) -> Reply {
        let script = match parse_script(text) {
            Ok(script) => script,
            Err(e) => return Reply::script_error(format!("script error: {e}")),
        };
        if script.seed.is_some() || script.size.is_some() {
            return Reply::script_error(
                "seed and size can't change mid-session; use new_game instead",
            );
        }
        if let Err(e) = validate_steps(&script.steps, &self.runner.app) {
            return Reply::script_error(format!("script error: {e}"));
        }
        let report = self.runner.run_steps(&script.steps);
        self.transcript.extend(
            report
                .completed
                .iter()
                .map(|l| l.step.clone())
                .filter(Step::changes_state),
        );
        Reply {
            status: report.status,
            output: report.output,
            error: report.error,
        }
    }

    /// The session as a headless script that replays to the same final state.
    pub fn export(&self) -> String {
        let Setup { seed, size, .. } = self.setup;
        let mut script = format!(
            "# Exported from an MCP session; replay with: just client-blind <this file>\n\
             seed {seed}\nsize {}x{}\n",
            size.0, size.1
        );
        for step in &self.transcript {
            script.push_str(&format!("{step}\n"));
        }
        script.push_str("state final\n");
        script
    }
}

enum Request {
    NewGame(Setup),
    Run(String),
    Export,
}

/// Handle to the session thread. Cloneable; requests are served one at a time.
#[derive(Clone)]
pub struct SessionHandle {
    tx: mpsc::Sender<(Request, oneshot::Sender<Reply>)>,
}

impl SessionHandle {
    /// Start the session thread. The session itself is created lazily on first use.
    pub fn spawn(default_seed: u64) -> Self {
        let (tx, rx) = mpsc::channel::<(Request, oneshot::Sender<Reply>)>();
        std::thread::Builder::new()
            .name("mcp-session".into())
            .spawn(move || {
                let default = Setup {
                    seed: default_seed,
                    size: DEFAULT_SIZE,
                    skip_splash: true,
                };
                let mut session: Option<Session> = None;
                for (request, reply_to) in rx {
                    let reply = match request {
                        Request::NewGame(setup) => {
                            session = Some(Session::new(setup));
                            Reply::ok(format!(
                                "new game: seed {} size {}x{}{}",
                                setup.seed,
                                setup.size.0,
                                setup.size.1,
                                if setup.skip_splash {
                                    ", splash skipped"
                                } else {
                                    ""
                                }
                            ))
                        }
                        Request::Run(text) => session
                            .get_or_insert_with(|| Session::new(default))
                            .run(&text),
                        Request::Export => Reply::ok(
                            session
                                .get_or_insert_with(|| Session::new(default))
                                .export(),
                        ),
                    };
                    let _ = reply_to.send(reply);
                }
            })
            .expect("spawn MCP session thread");
        Self { tx }
    }

    async fn request(&self, request: Request) -> Reply {
        let (reply_tx, reply_rx) = oneshot::channel();
        if self.tx.send((request, reply_tx)).is_err() {
            return Reply::script_error("game session stopped");
        }
        reply_rx
            .await
            .unwrap_or_else(|_| Reply::script_error("game session crashed"))
    }

    pub async fn new_game(&self, setup: Setup) -> Reply {
        self.request(Request::NewGame(setup)).await
    }

    pub async fn run(&self, text: impl Into<String>) -> Reply {
        self.request(Request::Run(text.into())).await
    }

    pub async fn export(&self) -> Reply {
        self.request(Request::Export).await
    }
}

// ---------------------------------------------------------------------------
// MCP tools
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NewGameArgs {
    /// World seed (default: the configured DEFAULT_SEED)
    pub seed: Option<u64>,
    /// Screen width in columns (default 160)
    pub cols: Option<u16>,
    /// Screen height in rows (default 50)
    pub rows: Option<u16>,
    /// Skip the splash screens (default true)
    pub skip_splash: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PressKeyArgs {
    /// Keybind as category.ACTION, e.g. movement.MOVE_SOUTH or action.MINE
    pub keybind: String,
    /// How many times to press it (default 1)
    pub count: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RawKeyArgs {
    /// Config key name: ENTER, ESCAPE, TAB, LEFT, F1, q, ...
    pub name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TextArgs {
    pub text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WaitArgs {
    /// Milliseconds of fake time to advance, in 50 ms frames
    pub ms: u64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TickArgs {
    /// Number of game ticks (default 1)
    pub n: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheatArgs {
    /// Cheat console command line, e.g. "tp 0 0 0" or "giveitem log 5"
    pub command: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ScriptArgs {
    /// Headless script steps, one per line (no seed/size lines)
    pub script: String,
}

#[derive(Clone)]
pub struct GameServer {
    session: SessionHandle,
    default_seed: u64,
    tool_router: ToolRouter<Self>,
}

/// Turn a session reply into a tool result: failures are tool errors, not protocol errors,
/// so the session keeps going.
fn to_result(reply: Reply) -> CallToolResult {
    match reply.error {
        None => CallToolResult::success(vec![ContentBlock::text(if reply.output.is_empty() {
            "ok".to_string()
        } else {
            reply.output
        })]),
        Some(error) => {
            let kind = if reply.status == EXIT_EXPECT_FAILED {
                "expectation failed"
            } else {
                "error"
            };
            let mut text = format!("{kind} (status {}):\n{error}", reply.status);
            if !reply.output.is_empty() {
                text.push_str(&format!("\noutput before the failure:\n{}", reply.output));
            }
            CallToolResult::error(vec![ContentBlock::text(text)])
        }
    }
}

/// Build a single script line from tool input, refusing input that would smuggle in
/// extra lines.
fn one_line(line: String) -> Result<String, CallToolResult> {
    if line.contains('\n') || line.contains('\r') {
        Err(CallToolResult::error(vec![ContentBlock::text(
            "arguments must not contain line breaks",
        )]))
    } else {
        Ok(line)
    }
}

#[tool_router(router = tool_router)]
impl GameServer {
    pub fn new(default_seed: u64) -> Self {
        Self {
            session: SessionHandle::spawn(default_seed),
            default_seed,
            tool_router: Self::tool_router(),
        }
    }

    async fn step(&self, line: String) -> Result<CallToolResult, ErrorData> {
        Ok(match one_line(line) {
            Ok(line) => to_result(self.session.run(line).await),
            Err(result) => result,
        })
    }

    #[tool(
        description = "Start a new game session, replacing the current one. Other tools start a default session (configured seed, 160x50, splash skipped) if none exists."
    )]
    async fn new_game(
        &self,
        Parameters(args): Parameters<NewGameArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        let setup = Setup {
            seed: args.seed.unwrap_or(self.default_seed),
            size: (
                args.cols.unwrap_or(DEFAULT_SIZE.0),
                args.rows.unwrap_or(DEFAULT_SIZE.1),
            ),
            skip_splash: args.skip_splash.unwrap_or(true),
        };
        Ok(to_result(self.session.new_game(setup).await))
    }

    #[tool(description = "Press the key bound to a keybind (category.ACTION), count times.")]
    async fn press_key(
        &self,
        Parameters(args): Parameters<PressKeyArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("key {} {}", args.keybind, args.count.unwrap_or(1)))
            .await
    }

    #[tool(description = "Press a key by its config name (ENTER, ESCAPE, TAB, LEFT, F1, q, ...).")]
    async fn raw_key(
        &self,
        Parameters(args): Parameters<RawKeyArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("rawkey {}", args.name)).await
    }

    #[tool(description = "Type text: one key press per character.")]
    async fn type_text(
        &self,
        Parameters(args): Parameters<TextArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("type \"{}\"", args.text)).await
    }

    #[tool(
        description = "Advance fake time by ms milliseconds (splash timers, combat auto-ticks)."
    )]
    async fn wait(
        &self,
        Parameters(args): Parameters<WaitArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("wait {}", args.ms)).await
    }

    #[tool(description = "Advance the game simulation by n ticks.")]
    async fn tick(
        &self,
        Parameters(args): Parameters<TickArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("tick {}", args.n.unwrap_or(1))).await
    }

    #[tool(
        description = "Run a cheat console command: tp x y z, giveitem <item> <qty>, noclip_toggle, toggle_fogofwar, toggle_tutorial."
    )]
    async fn cheat(
        &self,
        Parameters(args): Parameters<CheatArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("cheat {}", args.command)).await
    }

    #[tool(description = "Get the current screen as plain text.")]
    async fn screen(&self) -> Result<CallToolResult, ErrorData> {
        self.step("screen".to_string()).await
    }

    #[tool(
        description = "Get a JSON summary of the game: player, body, inventory, tab/modal, combat, nearby entities, recent messages."
    )]
    async fn state(&self) -> Result<CallToolResult, ErrorData> {
        self.step("state".to_string()).await
    }

    #[tool(
        description = "Check that the current screen contains text; returns an error result if not."
    )]
    async fn expect_text(
        &self,
        Parameters(args): Parameters<TextArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        self.step(format!("expect-text \"{}\"", args.text)).await
    }

    #[tool(
        description = "Run several headless script steps (one per line) against the current session; returns their output and status."
    )]
    async fn run_script(
        &self,
        Parameters(args): Parameters<ScriptArgs>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(to_result(self.session.run(args.script).await))
    }

    #[tool(
        description = "Export this session as a headless script (just client-blind) that replays to the same final state. Save it under scripts/ to make a finding reproducible."
    )]
    async fn export_script(&self) -> Result<CallToolResult, ErrorData> {
        Ok(to_result(self.session.export().await))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GameServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "lithicrivers",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Plays a headless LithicRivers session with a fake clock: same calls, same \
                 results. Start with new_game, act with press_key/cheat/wait/tick, observe \
                 with state (facts) and screen (layout). To make a finding reproducible, \
                 export_script and save it under scripts/.",
            )
    }
}

/// Serve MCP over stdin/stdout until the client disconnects.
pub fn serve_stdio(default_seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let service = GameServer::new(default_seed)
            .serve(rmcp::transport::stdio())
            .await?;
        service.waiting().await?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::headless::{run_script, Overrides};

    const SEED: u64 = 12345;

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(future)
    }

    fn text(result: &CallToolResult) -> String {
        result
            .content
            .iter()
            .filter_map(|c| c.as_text().map(|t| t.text.clone()))
            .collect()
    }

    fn is_error(result: &CallToolResult) -> bool {
        result.is_error == Some(true)
    }

    fn server() -> GameServer {
        GameServer::new(SEED)
    }

    async fn new_game(server: &GameServer, seed: u64) {
        let result = server
            .new_game(Parameters(NewGameArgs {
                seed: Some(seed),
                cols: Some(100),
                rows: Some(30),
                skip_splash: Some(true),
            }))
            .await
            .unwrap();
        assert!(!is_error(&result), "{}", text(&result));
    }

    fn state_seed(json_dump: &str) -> u64 {
        let json = json_dump
            .split_once("===\n")
            .and_then(|(_, rest)| rest.rsplit_once("\n==="))
            .map(|(json, _)| json)
            .expect("framed state dump");
        serde_json::from_str::<serde_json::Value>(json).unwrap()["seed"]
            .as_u64()
            .unwrap()
    }

    // --- 2.2: implicit session and replacement ---------------------------------------

    #[test]
    fn first_call_starts_a_default_session() {
        block_on(async {
            let server = server();
            let state = text(&server.state().await.unwrap());
            assert_eq!(state_seed(&state), SEED);
            assert!(
                state.contains("\"screen\": \"MainUI\""),
                "splash skipped by default"
            );
        });
    }

    #[test]
    fn new_game_replaces_the_session() {
        block_on(async {
            let server = server();
            server
                .press_key(Parameters(PressKeyArgs {
                    keybind: "movement.MOVE_SOUTH".into(),
                    count: Some(2),
                }))
                .await
                .unwrap();
            new_game(&server, 7).await;
            let state = text(&server.state().await.unwrap());
            assert_eq!(state_seed(&state), 7);
            let export = text(&server.export_script().await.unwrap());
            assert!(
                !export.contains("MOVE_SOUTH"),
                "old session's steps are gone: {export}"
            );
        });
    }

    // --- 2.3: tools behave exactly like script steps -----------------------------------

    #[test]
    fn tool_sequence_matches_the_equivalent_script() {
        let script = "seed 12345\nsize 100x30\nskip-splash\nkey movement.MOVE_SOUTH 2\n\
                      cheat tp 0 0 0\nkey action.MINE\nkey build.PLACE_EAST\nwait 200\ntick 2\nscreen\n";
        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        assert_eq!(
            run_script(script, 0, Overrides::default(), &mut out, &mut err_out),
            EXIT_OK
        );
        let expected = String::from_utf8(out).unwrap();

        let screen = block_on(async {
            let server = server();
            new_game(&server, SEED).await;
            for (keybind, count) in [("movement.MOVE_SOUTH", 2)] {
                server
                    .press_key(Parameters(PressKeyArgs {
                        keybind: keybind.into(),
                        count: Some(count),
                    }))
                    .await
                    .unwrap();
            }
            server
                .cheat(Parameters(CheatArgs {
                    command: "tp 0 0 0".into(),
                }))
                .await
                .unwrap();
            for keybind in ["action.MINE", "build.PLACE_EAST"] {
                server
                    .press_key(Parameters(PressKeyArgs {
                        keybind: keybind.into(),
                        count: None,
                    }))
                    .await
                    .unwrap();
            }
            server.wait(Parameters(WaitArgs { ms: 200 })).await.unwrap();
            server
                .tick(Parameters(TickArgs { n: Some(2) }))
                .await
                .unwrap();
            text(&server.screen().await.unwrap())
        });

        // The script labels its dump with a line number, the tool with "1"
        let body = |dump: &str| dump.split_once('\n').unwrap().1.to_string();
        assert_eq!(body(&screen), body(&expected));
    }

    // --- 2.4: errors are tool results and the session survives ------------------------

    #[test]
    fn bad_input_is_a_tool_error_and_the_session_continues() {
        block_on(async {
            let server = server();
            let bad = server
                .press_key(Parameters(PressKeyArgs {
                    keybind: "movement.FLY".into(),
                    count: None,
                }))
                .await
                .unwrap();
            assert!(is_error(&bad) && text(&bad).contains("movement.FLY"));

            let bad = server
                .cheat(Parameters(CheatArgs {
                    command: "summon_dragon".into(),
                }))
                .await
                .unwrap();
            assert!(is_error(&bad) && text(&bad).contains("summon_dragon"));

            let smuggled = server
                .cheat(Parameters(CheatArgs {
                    command: "tp 0 0 0\nquit".into(),
                }))
                .await
                .unwrap();
            assert!(is_error(&smuggled));

            let ok = server
                .press_key(Parameters(PressKeyArgs {
                    keybind: "movement.MOVE_SOUTH".into(),
                    count: None,
                }))
                .await
                .unwrap();
            assert!(!is_error(&ok), "{}", text(&ok));
            let failed = server
                .expect_text(Parameters(TextArgs {
                    text: "no such text on screen".into(),
                }))
                .await
                .unwrap();
            assert!(is_error(&failed) && text(&failed).contains("expectation failed"));
        });
    }

    #[test]
    fn run_script_reports_output_and_rejects_setup_lines() {
        block_on(async {
            let server = server();
            let result = server
                .run_script(Parameters(ScriptArgs {
                    script: "key movement.MOVE_SOUTH\nkey movement.MOVE_SOUTH\nstate after".into(),
                }))
                .await
                .unwrap();
            assert!(!is_error(&result) && text(&result).contains("=== state after ==="));

            let result = server
                .run_script(Parameters(ScriptArgs {
                    script: "seed 9\nkey movement.MOVE_SOUTH".into(),
                }))
                .await
                .unwrap();
            assert!(is_error(&result) && text(&result).contains("new_game"));
        });
    }

    // --- 3.1: export replays to the same final state ---------------------------------

    #[test]
    fn exported_session_replays_to_the_same_state() {
        let (export, session_state) = block_on(async {
            let server = server();
            new_game(&server, SEED).await;
            for step in [
                "key movement.MOVE_SOUTH 2",
                "screen",
                "cheat tp 0 0 0",
                "key action.MINE",
                "key build.PLACE_EAST",
                "key movement.FLY",
                "expect-text \"You chop the tree\"",
                "wait 500",
                "tick 3",
            ] {
                server
                    .run_script(Parameters(ScriptArgs {
                        script: step.into(),
                    }))
                    .await
                    .unwrap();
            }
            let export = text(&server.export_script().await.unwrap());
            let state = text(&server.state().await.unwrap());
            (export, state)
        });

        // Failed and read-only steps are not recorded
        assert!(
            !export.contains("FLY")
                && !export.contains("screen")
                && !export.contains("expect-text")
        );

        let (mut out, mut err_out) = (Vec::new(), Vec::new());
        let code = run_script(&export, 0, Overrides::default(), &mut out, &mut err_out);
        assert_eq!(code, EXIT_OK, "{}", String::from_utf8_lossy(&err_out));
        let replay = String::from_utf8(out).unwrap();
        let json = |dump: &str| dump.split_once('\n').unwrap().1.to_string();
        assert_eq!(json(&replay), json(&session_state));
    }
}
