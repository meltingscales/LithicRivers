#!/usr/bin/env bash
# PreToolUse hook: stop agents from launching the interactive TUI client.
# Reads the hook's JSON input on stdin; exit 2 blocks the call and shows stderr to the agent.
input=$(cat)

blocked=0
if grep -Eq 'just[[:space:]]+client-(release-)?(linux|windows)' <<<"$input"; then
    blocked=1
elif grep -Eq '(cargo[[:space:]]+run[^"]*lithicrivers-client|target/(debug|release)/lithicrivers-client)' <<<"$input" \
    && ! grep -q -- '--script' <<<"$input"; then
    blocked=1
fi

if [ "$blocked" -eq 1 ]; then
    echo "Blocked: the interactive client needs a real terminal. Use 'just client-blind <script>' instead (see AGENTS.md, 'Headless client'), or ask the user to run 'just client-linux'." >&2
    exit 2
fi
exit 0
