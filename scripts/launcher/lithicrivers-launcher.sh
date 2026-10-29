#!/usr/bin/env bash
# Starts LithicRivers from its install folder. The game opens its own window, so no
# terminal emulator is needed; pass --terminal to play in the current terminal instead.

log_file="lithicrivers-launcher-$(date +%Y-%m-%d_%H-%M-%S).log"
echo "Log file: $log_file"

echo_and_log_to_file() {
    local log_file="$1"
    shift
    echo "$@"
    echo "$@" >> "$log_file"
}

# The game reads its config and writes logs and saves next to itself
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo_and_log_to_file "$log_file" "SCRIPT_DIR=$SCRIPT_DIR"
GAME_BIN="$SCRIPT_DIR/lithicrivers-client"
echo_and_log_to_file "$log_file" "GAME_BIN=$GAME_BIN"
echo_and_log_to_file "$log_file" "GAME_ARGS=$*"

cd "$SCRIPT_DIR" || exit 1
exec "$GAME_BIN" "$@"
