# Spec Delta

## ADDED Requirements

### Requirement: Computers on the network
A SapienNet computer SHALL be a network member that reads the previous step's signals and writes outputs and enable states for the current step, like a logic block.

#### Scenario: Output signal
- **WHEN** a script emits `alarm = 1`
- **THEN** the network carries `alarm = 1` from the next step
