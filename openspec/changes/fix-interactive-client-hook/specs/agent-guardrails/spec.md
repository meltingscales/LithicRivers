# Spec Delta

## Purpose

Claude Code hooks that stop agents from actions they can't perform correctly in this repository, such as launching the interactive terminal client, while leaving every other command alone.

## ADDED Requirements

### Requirement: Block launching the interactive client
The hook SHALL block a Bash command when any segment it runs launches the interactive client: the `just` recipes `client-linux`, `client-windows`, `client-release-linux` or `client-release-windows`, or `cargo run` of `lithicrivers-client` / the built `lithicrivers-client` binary without `--script` or `--mcp` in that segment. The block message SHALL point to `just client-blind` and the MCP tools.

#### Scenario: Recipe launch
- **WHEN** an agent runs `cd repo && just client-linux`
- **THEN** the command is blocked with a message naming the headless alternatives

#### Scenario: Binary launch
- **WHEN** an agent runs `./target/debug/lithicrivers-client`
- **THEN** the command is blocked

#### Scenario: Exemption only applies to its own segment
- **WHEN** an agent runs `echo --script; cargo run -p lithicrivers-client --bin lithicrivers-client`
- **THEN** the command is blocked, because the launching segment has no `--script` or `--mcp`

### Requirement: Mentions are not launches
The hook SHALL NOT block a command that only mentions a launch: text inside heredoc bodies, quoted strings, comments, or arguments to other programs.

#### Scenario: PR body in a heredoc
- **WHEN** an agent runs `gh pr create --body-file - <<'EOF'` with a body that says "run just client-linux to check"
- **THEN** the command runs

#### Scenario: Quoted text
- **WHEN** an agent runs `git commit -m "Ask the user to run just client-linux"`
- **THEN** the command runs

#### Scenario: Searching for the recipe
- **WHEN** an agent runs `grep -n client-linux justfile`
- **THEN** the command runs

### Requirement: Headless and other commands are allowed
The hook SHALL allow headless runs (`just client-blind`, `just client-mcp`, the client with `--script` or `--mcp`), builds and tests of the client, and demo recipes.

#### Scenario: Headless script
- **WHEN** an agent runs `just client-blind scripts/agent/smoke.txt`
- **THEN** the command runs

#### Scenario: Build and test
- **WHEN** an agent runs `cargo build -p lithicrivers-client` or `cargo test -p lithicrivers-client`
- **THEN** the command runs

### Requirement: Fail open on unreadable input
If the hook cannot parse its input, it SHALL allow the command and print a warning to stderr, rather than block every Bash command.

#### Scenario: Malformed input
- **WHEN** the hook receives input that is not valid JSON
- **THEN** it exits successfully and prints a warning
