# Tasks

## 1. Deny rules

- [x] 1.1 Validate deny-rule matching with a sentinel rule (`Bash(echo zzprobe *)`): direct, `cd . &&`, newline, `FOO=1 timeout 5`, and `$(...)` are refused; heredoc and quoted mentions run
- [x] 1.2 Replace the hook registration in `.claude/settings.json` with the seven deny rules and remove `.claude/hooks/`; verify live that `./target/debug/lithicrivers-client --zzprobe`, `target/release/lithicrivers-client --zzprobe` and `cd . && cargo run -q -p lithicrivers-client --bin zz_missing_bin` are refused, a heredoc/grep mentioning the recipe runs, and `just client-blind scripts/agent/smoke.txt` exits 0

## 2. Docs

- [x] 2.1 Update AGENTS.md (Headless client section) to describe the deny rules, the generic refusal message, and that direct launches are refused even with `--script`; verify `cargo test --workspace` and `just code-clippy` pass
