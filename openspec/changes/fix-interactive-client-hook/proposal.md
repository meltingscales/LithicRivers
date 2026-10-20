# Proposal

## Why

The Claude Code `PreToolUse` hook in `.claude/hooks/block-interactive-client.sh` greps the hook's entire JSON input for the client launch text. Any Bash command that merely mentions the interactive client is blocked: a PR description in a heredoc, a commit message, a file written with `cat <<EOF`, a `grep` for the recipe name. This happened twice in one session (opening PR #62 and writing the `split-client-main` tasks), and each time the whole command was rejected and had to be redone with a workaround. The `--script` / `--mcp` exemption has the opposite problem: it also searches the whole input, so a command that launches the interactive client and mentions `--script` anywhere else is let through.

## What Changes

- The hook inspects only the `command` field of the Bash tool input and only the parts of it that run: each command segment (split on `;`, `&&`, `||`, `|`, newlines), with heredoc bodies, quoted strings and comments excluded.
- A segment is blocked only if its command is `just client-linux`, `just client-windows`, `just client-release-linux` / `-windows`, or runs the client (`cargo run ... lithicrivers-client` / the built binary) without `--script` or `--mcp` in that same segment.
- If the hook can't parse its input, it allows the command and writes a warning to stderr (blocking every command on a parse error would make the session unusable).
- Tests cover blocked and allowed cases so the hook can't regress silently.

## Capabilities

### New Capabilities
- `agent-guardrails`: Claude Code hooks that keep agents away from actions they can't perform correctly in this repo, starting with launching the interactive TUI.

### Modified Capabilities
<!-- None -->

## Impact

- `.claude/hooks/block-interactive-client.sh` (rewritten to call a small Python script that does the parsing; the hook warns and allows if `python3` is not installed).
- New test cases runnable with `cargo test` on Linux and macOS.
- No game code changes.
