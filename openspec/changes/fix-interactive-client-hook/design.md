# Design

## Context

- `.claude/settings.json` registers `bash "$CLAUDE_PROJECT_DIR/.claude/hooks/block-interactive-client.sh"` as a `PreToolUse` hook for the `Bash` tool. It receives JSON on stdin (`{"tool_name": "Bash", "tool_input": {"command": "..."}}`); exit 2 blocks and shows stderr to the agent.
- The current script greps the entire JSON input, so heredoc bodies, quoted text and file contents count as launches, and the `--script`/`--mcp` exemption is also checked against the whole input.
- `python3` and `jq` exist on the development machine. The hook only runs inside Claude Code sessions in this repo, never in CI or for players.

## Goals / Non-Goals

**Goals:**
- Block real launches, allow mentions.
- Keep the hook fast (it runs before every Bash call) and dependency-light.
- Cover it with tests.

**Non-Goals:**
- A full shell parser. Command substitution (`$(...)`), `eval`, and aliases are not analyzed; the hook is a guardrail against mistakes, not a security boundary.
- Hooks for other tools (Write, MCP).

## Decisions

### Parse in Python, keep the bash entry point
`block-interactive-client.sh` stays the registered command (no settings change) and runs `python3 .claude/hooks/block_interactive_client.py`. Python's `shlex` handles quoting correctly; doing the same in bash or `jq` would be fragile. If `python3` is missing, the shell script warns and allows (same fail-open rule as unparseable input).
Alternative: a Rust binary in the workspace. Rejected: the hook would need a build before it works and would slow every cold session.

### What counts as "runs"
1. Read `tool_input.command` from the JSON.
2. Remove heredoc bodies: for each `<<[-]['"]?WORD['"]?` operator, drop the following lines through the terminator line.
3. Tokenize the rest with `shlex` (`punctuation_chars=True`, comments on), which keeps quoted strings as single tokens and splits on `;`, `&&`, `||`, `|`, `&`.
4. Split tokens into segments at those operators and newlines, and skip leading `VAR=value` assignments in each segment. (`cd dir && ...` is already two segments.)
5. Block a segment if: its first two words are `just` + one of the four interactive recipes; or it contains `cargo` `run` and the token `lithicrivers-client` (as `-p`/`--bin` value), or its first word ends with `target/debug/lithicrivers-client` / `target/release/lithicrivers-client` (with or without `.exe`), and the segment has no `--script` or `--mcp` token.
Quoted arguments arrive as single tokens, so `"run just client-linux"` never matches as a command word.

### Tests run in `cargo test`
A client integration test (`crates/client/tests/agent_hook.rs`, `#[cfg(unix)]`) pipes JSON cases through the real hook script and checks the exit codes: every scenario in the spec plus the commands that were wrongly blocked this session. It runs on Linux and macOS in the PR checks; Windows runners are skipped because the hook targets the Claude Code environment, not players.

## Risks / Trade-offs

- [`python3` missing on some developer machine] → Fail open with a warning; nothing else breaks.
- [A cleverly hidden launch slips through (`$(...)`, `eval`)] → Accepted; agents aren't adversarial here, and the AGENTS.md rule still applies.
- [shlex can't parse an unbalanced quote mid-edit] → Fail open with a warning for that command.
