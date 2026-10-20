# Proposal

## Why

The Claude Code `PreToolUse` hook in `.claude/hooks/block-interactive-client.sh` greps the hook's entire JSON input for the client launch text. Any Bash command that merely mentions the interactive client is blocked: a PR description in a heredoc, a commit message, a file written with `cat <<EOF`, a `grep` for the recipe name. This happened four times in one session, and each time the whole command was rejected and had to be redone with a workaround. The `--script` / `--mcp` exemption has the opposite problem: it also searches the whole input, so a command that launches the interactive client and mentions `--script` anywhere else is let through.

## What Changes

- Replace the hook with Claude Code permission **deny rules** in `.claude/settings.json`. Claude Code matches them against each subcommand it parses from the real command (splitting `&&`, `;`, `|`, newlines, subshells and `$(...)`, stripping wrappers like `timeout` and leading `VAR=value`), so mentions in heredocs, quoted strings and arguments to other programs don't match.
- Rules: the four interactive `just` recipes, `cargo run` of `lithicrivers-client`, and running `target/*/lithicrivers-client` directly. Direct launches are blocked even with `--script`/`--mcp`: agents use `just client-blind` and `just client-mcp`, and `.mcp.json` launches the MCP server outside the Bash tool.
- Remove `.claude/hooks/` (bash wrapper and Python tokenizer).

An earlier version of this change implemented a Python tokenizer hook with a 24-case test; deny rules were tested live against the same cases and replace it because they are simpler and also catch launches inside `$(...)`, which the hook could not.

## Capabilities

### New Capabilities
- `agent-guardrails`: Claude Code settings that keep agents away from actions they can't perform correctly in this repo, starting with launching the interactive TUI.

### Modified Capabilities
<!-- None -->

## Impact

- `.claude/settings.json` (hook registration replaced by `permissions.deny`), `.claude/hooks/` removed, AGENTS.md note.
- Trade-offs: the block message is Claude Code's generic "Permission ... denied" (AGENTS.md explains why and what to use instead); running the client binary directly with `--script` is no longer allowed for agents. Deny rules can only be verified inside Claude Code, so there is no `cargo test` for them.
- No game code changes.
