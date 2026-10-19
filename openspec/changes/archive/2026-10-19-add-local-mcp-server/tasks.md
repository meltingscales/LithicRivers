# Tasks

## 1. Runner groundwork

- [x] 1.1 Expose a single-step executor on the headless runner that returns output as a string and reports errors as values; verify existing headless script tests still pass
- [x] 1.2 Add rendering of `Step` values back to script syntax; verify a round-trip test (parse → render → parse) on every step kind

## 2. Server

- [x] 2.1 Add `rmcp` and `tokio` (current_thread) to the client, pinned; add `--mcp` flag in `main()` that starts the stdio server; verify `cargo build --workspace` succeeds and `just code-clippy` passes
- [x] 2.2 Implement the session holder (mutex-guarded headless `App` + backend, implicit default session, `new_game` replacement); verify unit tests for implicit start and replacement
- [x] 2.3 Implement tools for new game, keybind press, raw key, type text, wait, tick, cheat, screen, state, expect text, and run script fragment, each delegating to the step executor; verify a test that a tool sequence's screen output equals the equivalent script's output
- [x] 2.4 Return tool errors (unknown keybind, bad cheat) as MCP error results without ending the session; verify a test calling an unknown keybind then a valid one

## 3. Recording and export

- [x] 3.1 Record successful state-changing steps with session setup; implement the export tool; verify a test that replaying the exported script yields the same final state dump as the session
- [x] 3.2 Verify stdout hygiene with a test that a tool sequence emits only protocol messages

## 4. Integration and docs

- [x] 4.1 Add `just client-mcp` and a project `.mcp.json`; verify Claude Code lists the server's tools after restart (user check) and a manual new game → screen call works
- [x] 4.2 Update AGENTS.md: use scripts for reproducible checks, MCP for exploration, and export MCP findings to `scripts/` before claiming a bug is reproduced; verify the documented recipe names exist
