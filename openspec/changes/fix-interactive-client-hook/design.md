# Design

## Context

- `.claude/settings.json` registered `.claude/hooks/block-interactive-client.sh` as a `PreToolUse` hook for Bash. It grepped the whole hook JSON, causing false blocks on mentions and a false allow when `--script` appeared in another subcommand.
- Claude Code permission rules (`permissions.deny`) match Bash commands per subcommand after parsing the command: they split on `&&`, `||`, `;`, `|`, `|&`, `&` and newlines, check commands nested in subshells, command substitutions and control-flow bodies, strip wrappers (`timeout`, `time`, `nice`, `nohup`, `stdbuf`, `command`, `builtin`, `noglob`) and, for deny rules, any leading variable assignment. A `*` matches any text and may appear anywhere in a rule. Rules match the command text as written, not the program by path.

## Goals / Non-Goals

**Goals:**
- Refuse real launches of the interactive client; never refuse mentions.
- As little custom machinery as possible.

**Non-Goals:**
- A security boundary. `bash -c '...'`, an absolute binary path, or a shell alias aren't matched; agents aren't adversarial here, and AGENTS.md states the rule.
- A custom refusal message.

## Decisions

### Deny rules instead of a hook
Seven rules in `.claude/settings.json`:

```
Bash(just client-linux*)          Bash(just client-windows*)
Bash(just client-release-linux*)  Bash(just client-release-windows*)
Bash(cargo run *lithicrivers-client*)
Bash(./target/*/lithicrivers-client*)  Bash(target/*/lithicrivers-client*)
```

Alternative 1 (implemented first): a Python tokenizer hook using `shlex` with heredoc stripping and a 24-case `cargo test`. It worked but re-implemented shell parsing, missed launches inside `$(...)`, and needed two scripts. Rejected in favor of the built-in parser.
Alternative 2: deny rules that end at the client name (`Bash(cargo run *lithicrivers-client)`) to keep allowing `-- --script`. Tested live: it misses launches with trailing flags such as `--release`. Rejected; headless runs go through `just client-blind` / `just client-mcp` instead.

### Binary rules anchored at the start
`./target/*/lithicrivers-client*` and `target/*/lithicrivers-client*` rather than `*target/*/lithicrivers-client*`, so `ls target/debug/lithicrivers-client` or `file ...` aren't refused.

### Verification
Deny rules only run inside Claude Code, so verification is live: harmless commands that match each rule shape (an unknown flag makes the binary exit before touching the terminal; `cargo run` of a missing binary errors) are refused, mention cases run, and `just client-blind` runs. The same shapes were first validated with a sentinel rule (`Bash(echo zzprobe *)`) covering direct, compound, newline, `VAR=` + `timeout`, `$(...)`, heredoc and quoted cases.

## Risks / Trade-offs

- [Generic refusal message] → AGENTS.md explains what is blocked and what to use instead.
- [No automated regression test] → Rules are short and declarative; the verification steps are listed in tasks.md for re-running if they change.
- [Developers outside Claude Code] → Unaffected; these settings only apply to Claude Code sessions in this repo.
