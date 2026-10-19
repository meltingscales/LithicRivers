# Spec Delta

## Purpose

Lets an AI agent play and inspect a live headless game session through MCP tools, and turn that session into a reproducible headless script.

## ADDED Requirements

### Requirement: Local stdio server
The client SHALL provide an MCP server mode that communicates only over stdin/stdout, opens no network sockets, requires no authentication, never touches the terminal or audio device, and writes nothing but protocol messages to stdout.

#### Scenario: Start from Claude Code
- **WHEN** Claude Code launches the server from the repository's `.mcp.json`
- **THEN** the server completes MCP initialization and lists its tools

#### Scenario: No network exposure
- **WHEN** the server is running
- **THEN** it has no listening network sockets

### Requirement: Game session tools
The server SHALL expose tools equivalent to the headless script steps: new game (seed, screen size, skip splash), keybind press with count, raw key, type text, wait milliseconds, tick, cheat command, screen, state, and expect text. Each SHALL behave exactly like the matching script step.

#### Scenario: Move and look
- **WHEN** an agent calls new game with seed 12345, then press keybind `movement.MOVE_NORTH`, then screen
- **THEN** the screen result matches what a headless script with the same steps prints for `screen`

#### Scenario: Tool error
- **WHEN** an agent presses an unknown keybind
- **THEN** the tool returns an error result naming the keybind and the session continues

### Requirement: Implicit session
The server SHALL hold one game session. Calls before a new game SHALL start one with the configured default seed, the default screen size, and the splash screens skipped. Calling new game SHALL replace the current session and its recorded steps.

#### Scenario: Screen before new game
- **WHEN** an agent calls screen as its first tool call
- **THEN** a session with the default seed is started and its main game screen is returned

#### Scenario: Replacing the session
- **WHEN** an agent moves the player, then calls new game with a different seed, then exports
- **THEN** the state reports the new seed and the export contains none of the earlier moves

### Requirement: Run script fragment
The server SHALL provide a tool that runs multi-line headless script text against the current session and returns its output and status.

#### Scenario: Batch steps
- **WHEN** an agent runs a fragment with three `key` steps and a `state` step
- **THEN** the result contains the state dump and status 0

#### Scenario: Setup lines in a fragment
- **WHEN** a fragment contains a `seed` or `size` line
- **THEN** the tool returns an error pointing to new game, and the session is unchanged

### Requirement: Session export
Every successful state-changing call SHALL be recorded as equivalent script lines. An export tool SHALL return the session as a headless script that reproduces the same final state.

#### Scenario: Replay exported session
- **WHEN** an agent plays a session, exports it, and the exported script runs with `just client-blind`
- **THEN** the script's final `state` dump equals the session's final state

### Requirement: Shared determinism
Server sessions SHALL use the same fake clock and determinism guarantees as headless scripts; tool calls SHALL never advance game time based on real elapsed time.

#### Scenario: Slow agent
- **WHEN** an agent waits a minute in real time between two tool calls
- **THEN** game time and state do not change between those calls
