#!/usr/bin/env python3
"""PreToolUse hook logic: block Bash commands that launch the interactive TUI client.

Reads the hook's JSON input on stdin. Only the parts of `tool_input.command` that would
actually run are checked: heredoc bodies and quoted strings are ignored, so a command that
merely mentions a launch (PR text, commit messages, grep patterns) is allowed.

Exit 2 blocks the call and shows stderr to the agent; exit 0 allows it. Anything that can't
be parsed is allowed with a warning, so a parsing problem never blocks every command.
This is a guardrail against mistakes, not a security boundary: `$(...)`, `eval` and
`bash -c "..."` are not analyzed.
"""

import json
import os
import re
import shlex
import sys

INTERACTIVE_RECIPES = {
    "client-linux",
    "client-windows",
    "client-release-linux",
    "client-release-windows",
}
CLIENT_BINARIES = {"lithicrivers-client", "lithicrivers-client.exe"}
HEADLESS_FLAGS = {"--script", "--mcp"}
# Tokens that end one command and start another
SEPARATORS = {";", "&&", "||", "|", "|&", "&", "\n", "(", ")"}
# Words that run the command after them
PREFIX_COMMANDS = {"env", "time", "nohup", "exec", "command", "nice"}
ASSIGNMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*=")
HEREDOC = re.compile(r"<<-?\s*(['\"]?)([A-Za-z_][A-Za-z0-9_]*)\1")

MESSAGE = (
    "Blocked: the interactive client needs a real terminal. Use 'just client-blind <script>' "
    "or the lithicrivers MCP tools instead (see AGENTS.md, 'Headless client'), or ask the "
    "user to run 'just client-linux'."
)


def strip_heredoc_bodies(command):
    """Drop the body lines of every heredoc, keeping the line that opens it."""
    kept = []
    terminators = []
    for line in command.split("\n"):
        if terminators:
            if line.strip() == terminators[0]:
                terminators.pop(0)
            continue
        kept.append(line)
        terminators.extend(match.group(2) for match in HEREDOC.finditer(line))
    return "\n".join(kept)


def segments(command):
    """Split a command into the word lists of the simple commands it runs."""
    lexer = shlex.shlex(
        strip_heredoc_bodies(command), posix=True, punctuation_chars="();<>|&\n"
    )
    lexer.whitespace = " \t\r"
    lexer.whitespace_split = True
    lexer.commenters = ""
    current = []
    for token in lexer:
        if token in SEPARATORS:
            if current:
                yield current
            current = []
        else:
            current.append(token)
    if current:
        yield current


def command_words(words):
    """Skip leading `VAR=value` assignments and prefixes like `env` or `time`."""
    i = 0
    while i < len(words) and (ASSIGNMENT.match(words[i]) or words[i] in PREFIX_COMMANDS):
        i += 1
    return words[i:]


def launches_interactive_client(words):
    words = command_words(words)
    if not words:
        return False
    program = os.path.basename(words[0])
    if program == "just":
        recipe = next((w for w in words[1:] if not w.startswith("-")), None)
        return recipe in INTERACTIVE_RECIPES
    runs_client = program in CLIENT_BINARIES or (
        program == "cargo" and "run" in words and "lithicrivers-client" in words
    )
    return runs_client and not HEADLESS_FLAGS.intersection(words)


def main():
    try:
        payload = json.load(sys.stdin)
        if payload.get("tool_name", "Bash") != "Bash":
            return 0
        command = payload["tool_input"]["command"]
        blocked = any(launches_interactive_client(words) for words in segments(command))
    except Exception as error:  # fail open: never block every command on a parse problem
        print(f"block_interactive_client: could not check command ({error}); allowing it", file=sys.stderr)
        return 0
    if blocked:
        print(MESSAGE, file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
