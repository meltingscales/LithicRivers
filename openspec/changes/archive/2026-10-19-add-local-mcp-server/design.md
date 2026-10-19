# Design

## Context

- After `add-headless-client-driver`: a headless `App` with `Clock::Fake`, a parsed `Step` enum, a runner that executes steps and produces framed screen/state text, and `--script` handling in `main()`.
- The client is synchronous; ratatui rendering and `App` are single-threaded.
- Claude Code reads project MCP servers from `.mcp.json` at the repository root.

## Goals / Non-Goals

**Goals:**
- Tools are a thin layer over the existing step executor; no game logic in the MCP module.
- Sessions can always be turned into committed scripts.

**Non-Goals:**
- Multiple concurrent sessions, HTTP/SSE transport, remote access, auth.
- MCP resources or prompts; tools only.

## Decisions

### `rmcp` over a hand-rolled JSON-RPC loop
`rmcp` is the official Rust MCP SDK and handles initialization, protocol version negotiation, tool schemas, and error shapes. A hand-rolled stdio loop is ~200 lines but has to track protocol revisions. Cost: `rmcp` + `tokio` in the client. Mitigation: MCP code lives in its own module and only runs under `--mcp`; use a `current_thread` runtime. Pinned at implementation time: `rmcp` 3.5.1 (features `server`, `transport-io`, `macros`), `tokio` 1.54 (`rt`, `macros`, `io-std`, `sync`), `schemars` 1.2 for tool argument schemas.

### Tools map 1:1 to `Step`
Each tool formats its arguments as one script line (rejecting line breaks) and runs it through the same parser, validator and `Runner::run_steps` executor as `just client-blind`. Failures come back as MCP tool errors (`isError`), not protocol errors, so the session continues. One implementation, so MCP and scripts can't diverge, and the recorded transcript is just those steps rendered back to script syntax.

### One session on a dedicated thread
The headless `App` isn't `Send` (rodio's `OutputStream` type is `!Send` even when audio is disabled), so it can't sit behind a mutex in the async handler. Instead a dedicated thread owns the session and serves requests from an `mpsc` channel, one at a time; tool handlers send script text and await a `oneshot` reply. Same behavior as the mutex plan (one session, requests serialized), without an `unsafe impl Send`.

### Recording and export
Each successful state-changing step is appended to a `Vec<Step>` together with the session's setup (`seed`, `size`, `skip-splash`). Read-only tools (`screen`, `state`) are not recorded, except that export appends a final `state` so a replay ends by printing it. `new_game` clears the transcript.

### Stdout hygiene
In `--mcp` mode, tracing goes to the log file only, and the runner's dump functions return strings instead of printing. Any accidental `println!` would corrupt the protocol, so the MCP module writes through the transport only; add a test that runs a tool sequence and checks nothing else was printed (using a captured stdout in-process).

### Launch
`just client-mcp` runs `cargo run -q -p lithicrivers-client -- --mcp`. `.mcp.json` runs the same command so Claude Code builds and starts it on demand; cargo's build output goes to stderr and doesn't touch the protocol stream.

## Risks / Trade-offs

- [Dependency weight / build time] → Isolated module; acceptable for a dev tool. Could move behind a cargo feature later if release builds grow noticeably.
- [`cargo run` in `.mcp.json` is slow on a cold cache, causing MCP startup timeouts] → Document running `just build-linux` first; the recipe can use the prebuilt binary path instead if needed.
- [Agents treat MCP sessions as proof] → AGENTS.md: a bug found via MCP must be exported to `scripts/` (or a snapshot script) before it counts as reproduced.
