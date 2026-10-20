#!/usr/bin/env bash
# PreToolUse hook: stop agents from launching the interactive TUI client.
# The parsing lives in block_interactive_client.py (only real launches are blocked, not
# mentions in heredocs or quoted text). Exit 2 blocks the call and shows stderr to the agent.
script="${0%/*}/block_interactive_client.py"

if ! command -v python3 >/dev/null 2>&1; then
    echo "block-interactive-client: python3 not found; not checking this command" >&2
    exit 0
fi

exec python3 "$script"
