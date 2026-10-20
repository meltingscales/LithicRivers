# Tasks

## 1. Hook

- [x] 1.1 Add `.claude/hooks/block_interactive_client.py`: read the JSON, take `tool_input.command`, strip heredoc bodies, tokenize with `shlex`, split into segments and apply the block rules from the design; exit 2 with the existing message on a block, warn and exit 0 on unparseable input; verify by piping each spec scenario through it by hand
- [x] 1.2 Make `block-interactive-client.sh` call the Python script (warn and allow if `python3` is missing); verify the registered hook still blocks a real recipe launch and allows `just client-blind scripts/agent/smoke.txt`

## 2. Tests and docs

- [x] 2.1 Add `crates/client/tests/agent_hook.rs` (`#[cfg(unix)]`) running every spec scenario, plus this session's false positives (heredoc PR body, the `split-client-main` tasks file written with `cat <<EOF`), through the hook script; verify `cargo test --workspace` passes and that the test fails against the old script
- [x] 2.2 Mention in AGENTS.md (Headless client section) that the hook blocks only real launches; verify `just code-clippy` passes
