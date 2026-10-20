//! The Claude Code hook in `.claude/hooks/` must block commands that launch the interactive
//! client and allow everything else, including commands that only mention a launch.
//! Unix only: the hook runs in the Claude Code environment, not on players' machines.
#![cfg(unix)]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn hook_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../.claude/hooks/block-interactive-client.sh")
}

/// Exit status of the hook for a Bash command (2 = blocked, 0 = allowed).
fn hook_status(stdin: &str) -> i32 {
    let mut child = Command::new("bash")
        .arg(hook_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("run hook");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child
        .wait()
        .expect("wait for hook")
        .code()
        .expect("hook exited normally")
}

fn bash(command: &str) -> i32 {
    hook_status(
        &serde_json::json!({"tool_name": "Bash", "tool_input": {"command": command}}).to_string(),
    )
}

const BLOCKED: &[&str] = &[
    "cd repo && just client-linux",
    "just client-windows",
    "just client-release-linux",
    "./target/debug/lithicrivers-client",
    "target/release/lithicrivers-client.exe",
    "cargo run -p lithicrivers-client --bin lithicrivers-client",
    // The headless exemption only counts in the segment that launches the client
    "echo --script; cargo run -p lithicrivers-client --bin lithicrivers-client",
    "cargo build -q && RUST_LOG=info just client-linux",
    "ls\njust client-linux",
    "echo hi # note\njust client-linux",
];

const ALLOWED: &[&str] = &[
    // Mentions are not launches (both were wrongly blocked before this fix)
    "gh pr create --body-file - <<'EOF'\nPlease run just client-linux to check.\nEOF",
    "cat > tasks.md <<EOF\n- [ ] 4.2 User check: just client-linux looks the same\nEOF\nopenspec validate x",
    "git commit -m \"Ask the user to run just client-linux\"",
    "grep -n client-linux justfile",
    "python3 - <<'EOF'\ns = 'just client-linux'\nEOF",
    // Headless runs, builds, tests and demos
    "just client-blind scripts/agent/smoke.txt",
    "just client-mcp",
    "cargo run -q -p lithicrivers-client --bin lithicrivers-client -- --script x.txt",
    "cargo run -q -p lithicrivers-client --bin lithicrivers-client -- --mcp",
    "./target/debug/lithicrivers-client --script s.txt > out.txt 2>&1 </dev/null",
    "cargo build -p lithicrivers-client",
    "cargo test -p lithicrivers-client",
    "just demo-crafting",
    // Unparseable commands fail open
    "echo 'unbalanced",
];

#[test]
fn hook_blocks_real_launches() {
    for command in BLOCKED {
        assert_eq!(bash(command), 2, "should block: {command:?}");
    }
}

#[test]
fn hook_allows_mentions_and_headless_commands() {
    for command in ALLOWED {
        assert_eq!(bash(command), 0, "should allow: {command:?}");
    }
}

#[test]
fn hook_fails_open_on_malformed_input() {
    assert_eq!(hook_status("not json"), 0);
}
