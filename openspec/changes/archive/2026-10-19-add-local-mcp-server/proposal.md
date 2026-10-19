# Proposal

## Why

Headless scripts (from `add-headless-client-driver`) suit reproducible checks, but exploratory play ("go find bugs in crafting") needs an agent to look at the screen, decide, and act in a loop. A local MCP server lets Claude Code drive a live headless game session through tools, and record the session as a replayable script.

## What Changes

- Add an MCP server mode to the client (`just client-mcp`), using stdio transport only, no network listener, and no authentication.
- Expose tools mirroring script steps: start a new game, press keybinds and raw keys, type text, wait (fake time), tick, run a cheat command, get the screen, get the state snapshot, check expected text, and run a script fragment.
- Record every state-changing tool call as script lines; a tool returns the session as a headless script so a found bug can be committed as a reproduction.
- Register the server for this repository in a project `.mcp.json`.
- Add the `rmcp` crate (and a `tokio` single-thread runtime) to the client, used only in MCP mode.

## Capabilities

### New Capabilities
- `agent-mcp-server`: A local stdio MCP server that drives a headless game session through tools and can export that session as a headless script.

### Modified Capabilities
<!-- None: no existing specs. `headless-client` and `game-state-snapshot` are introduced by add-headless-client-driver and reused unchanged. -->

## Impact

- Depends on `add-headless-client-driver` (step executor, clock, screen and state dumps).
- `crates/client`: new MCP module, `--mcp` flag, `rmcp` and `tokio` dependencies.
- `justfile`: `client-mcp` recipe. Repository: `.mcp.json`. AGENTS.md: when to use MCP vs scripts.
- Larger client dependency tree and build time.
