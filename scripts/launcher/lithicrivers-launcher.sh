#!/usr/bin/env bash

log_file="lithicrivers-launcher-$(date +%Y-%m-%d_%H-%M-%S).log"
echo "Log file: $log_file"

echo_and_log_to_file() {
    local log_file="$1"
    shift
    echo "$@"
    echo "$@" >> "$log_file"
}

# Function to detect if we're in a desktop environment
is_desktop_environment() {
    # Check for X11 or Wayland display server
    [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ] && return 0
    
    # Check if we're in an SSH session
    [ -n "$SSH_CONNECTION" ] || [ -n "$SSH_TTY" ] && return 1
    
    # Check if we're on a TTY (not a serial console)
    tty -s && [ "$(tty)" != "/dev/tty1" ] && [ "$(tty)" != "/dev/console" ] && return 0
    
    return 1
}

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo_and_log_to_file "$log_file" "SCRIPT_DIR=$SCRIPT_DIR"
GAME_BIN="$SCRIPT_DIR/lithicrivers-client"
echo_and_log_to_file "$log_file" "GAME_BIN=$GAME_BIN"
GAME_ARGS="$@"
echo_and_log_to_file "$log_file" "GAME_ARGS=$GAME_ARGS"

# If not in a desktop environment, try to run directly
if ! is_desktop_environment; then
    echo_and_log_to_file "$log_file" "Running game directly"
    exec "$GAME_BIN" $GAME_ARGS
    exit $?
fi

# Try to find a suitable terminal emulator
TERMINAL=""
for term in x-terminal-emulator terminator konsole gnome-terminal xfce4-terminal lxterminal xterm; do
    if command -v "$term" >/dev/null 2>&1; then
        TERMINAL="$term"
        break
    fi
done

if [ -z "$TERMINAL" ]; then
    echo_and_log_to_file "$log_file" "No terminal emulator found, running game directly"
    exec "$GAME_BIN" $GAME_ARGS
else
    echo_and_log_to_file "$log_file" "Found terminal emulator: $TERMINAL"
    # Run the game in the found terminal
    case "$TERMINAL" in
        terminator)
            echo_and_log_to_file "$log_file" "Running game in terminator"
            exec "$TERMINAL" -x bash -c "cd \"$SCRIPT_DIR\" && \"$GAME_BIN\" $GAME_ARGS; bash"
            ;;
        gnome-terminal|xfce4-terminal|konsole|lxterminal)
            echo_and_log_to_file "$log_file" "Running game in $TERMINAL"
            exec "$TERMINAL" -e "cd \"$SCRIPT_DIR\" && \"$GAME_BIN\" $GAME_ARGS"
            ;;
        *)
            echo_and_log_to_file "$log_file" "Running game in $TERMINAL"
            exec "$TERMINAL" -e "cd \"$SCRIPT_DIR\" && \"$GAME_BIN\" $GAME_ARGS; echo '[LithicRivers Unix Launcher] Press Enter to close...'; read"
            ;;
    esac
fi