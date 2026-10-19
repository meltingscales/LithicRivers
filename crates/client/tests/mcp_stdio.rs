//! Runs the real client binary in `--mcp` mode and checks that stdout carries nothing but
//! JSON-RPC messages, even across errors and screen/state dumps.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};

#[test]
fn mcp_stdout_is_only_protocol_messages() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lithicrivers-client"))
        .arg("--mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("start client in --mcp mode");

    let call = |id: u64, name: &str, args: serde_json::Value| {
        serde_json::json!({"jsonrpc": "2.0", "id": id, "method": "tools/call",
            "params": {"name": name, "arguments": args}})
    };
    let messages = [
        serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
            "protocolVersion": "2025-06-18", "capabilities": {},
            "clientInfo": {"name": "stdout-hygiene-test", "version": "0"}}}),
        serde_json::json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
        call(
            3,
            "new_game",
            serde_json::json!({"seed": 12345, "cols": 100, "rows": 30}),
        ),
        call(
            4,
            "press_key",
            serde_json::json!({"keybind": "movement.MOVE_SOUTH", "count": 2}),
        ),
        call(
            5,
            "press_key",
            serde_json::json!({"keybind": "movement.FLY"}),
        ),
        call(6, "screen", serde_json::json!({})),
        call(7, "state", serde_json::json!({})),
        call(8, "export_script", serde_json::json!({})),
    ];

    let mut stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    for message in &messages {
        writeln!(stdin, "{message}").unwrap();
    }
    stdin.flush().unwrap();

    let mut ids = Vec::new();
    let mut lines = stdout.lines();
    while ids.len() < 8 {
        let line = lines
            .next()
            .expect("server closed stdout early")
            .expect("read stdout");
        let value: serde_json::Value = serde_json::from_str(&line)
            .unwrap_or_else(|e| panic!("non-JSON line on stdout ({e}): {line:?}"));
        assert_eq!(value["jsonrpc"], "2.0", "not JSON-RPC: {line}");
        if let Some(id) = value["id"].as_u64() {
            assert!(value.get("error").is_none(), "protocol error: {line}");
            ids.push(id);
        }
    }
    ids.sort();
    assert_eq!(ids, vec![1, 2, 3, 4, 5, 6, 7, 8]);

    // Closing stdin ends the session; nothing else may appear on stdout
    drop(stdin);
    for line in lines {
        let line = line.expect("read stdout");
        assert!(
            serde_json::from_str::<serde_json::Value>(&line).is_ok(),
            "non-JSON line on stdout after shutdown: {line:?}"
        );
    }
    assert!(child.wait().expect("wait for client").success());
}
