# agent-guardrails Specification

## Purpose

Claude Code settings that stop agents from actions they can't perform correctly in this repository, such as launching the interactive terminal client, while leaving every other command alone.

## Requirements

### Requirement: Block launching the interactive client
Claude Code SHALL refuse a Bash command when any subcommand launches the interactive client: the `just` recipes `client-linux`, `client-windows`, `client-release-linux` or `client-release-windows`, `cargo run` of `lithicrivers-client`, or running a `target/<profile>/lithicrivers-client` binary directly. This applies wherever the subcommand appears: after `&&`, `;`, `|` or a newline, inside `$(...)` or a subshell, and behind wrappers such as `timeout` or a leading `VAR=value`.

#### Scenario: Recipe launch in a compound command
- **WHEN** an agent runs `cd repo && just client-linux`
- **THEN** the command is refused

#### Scenario: Binary launch
- **WHEN** an agent runs `./target/debug/lithicrivers-client` or `target/release/lithicrivers-client`
- **THEN** the command is refused

#### Scenario: Launch inside a command substitution
- **WHEN** an agent runs `true "$(just client-linux)"`
- **THEN** the command is refused

### Requirement: Direct launches are refused even when headless
Running the client through `cargo run` or the binary path SHALL be refused even with `--script` or `--mcp`. Agents SHALL use `just client-blind <script>` and `just client-mcp` for headless runs.

#### Scenario: Direct headless launch
- **WHEN** an agent runs `cargo run -q -p lithicrivers-client --bin lithicrivers-client -- --script x.txt`
- **THEN** the command is refused, and `just client-blind x.txt` is the allowed way to run it

### Requirement: Mentions are not launches
Commands that only mention a launch SHALL NOT be refused: text inside heredoc bodies, quoted strings, or arguments to other programs.

#### Scenario: PR body in a heredoc
- **WHEN** an agent runs `gh pr create --body-file - <<'EOF'` with a body that says "run just client-linux to check"
- **THEN** the command runs

#### Scenario: Quoted text and searches
- **WHEN** an agent runs `git commit -m "Ask the user to run just client-linux"` or `grep -n client-linux justfile`
- **THEN** the command runs

### Requirement: Headless recipes, builds and tests are allowed
`just client-blind`, `just client-mcp`, and building or testing the client SHALL be allowed.

#### Scenario: Headless script
- **WHEN** an agent runs `just client-blind scripts/agent/smoke.txt`
- **THEN** the command runs

#### Scenario: Build and test
- **WHEN** an agent runs `cargo build -p lithicrivers-client` or `cargo test -p lithicrivers-client`
- **THEN** the command runs
