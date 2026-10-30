# Spec Delta

## MODIFIED Requirements

### Requirement: Interactive client unchanged
Running the client without a script SHALL behave as before apart from where it draws: real time, audio, and the same screens and input. By default it SHALL open the game window (`game-window` spec); with `--terminal` it SHALL draw in the terminal as before. Headless runs SHALL keep drawing to an in-memory screen, so their output does not depend on which frontend the interactive client uses.

#### Scenario: Normal launch
- **WHEN** a player runs `just client-linux`
- **THEN** splash timing, combat auto-ticks, music, and input behave as they did before this change, in the game window

#### Scenario: Terminal launch
- **WHEN** a player runs the client with `--terminal`
- **THEN** it draws in the terminal exactly as before this change

#### Scenario: Headless output unchanged
- **WHEN** the snapshot scripts run after the game window is added
- **THEN** their output matches the stored snapshots
