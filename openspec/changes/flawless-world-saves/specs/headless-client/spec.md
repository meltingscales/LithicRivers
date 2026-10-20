# Spec Delta

## ADDED Requirements

### Requirement: Save-reload step
`save-reload` SHALL save the running game (including client play state) to a temporary location and load it back into the same session, then continue the script. It SHALL fail the run with status 1 if saving or loading fails.

#### Scenario: Transparent reload
- **WHEN** a script runs `key movement.MOVE_SOUTH`, `save-reload`, `key movement.MOVE_SOUTH`, `state`
- **THEN** the state dump is identical to the same script without the `save-reload` line

#### Scenario: Reload through MCP
- **WHEN** an MCP client runs `save-reload` via the `run_script` tool
- **THEN** the session continues from the reloaded state and the step is included in `export_script`
