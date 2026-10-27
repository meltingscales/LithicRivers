# Spec Delta

## ADDED Requirements

### Requirement: Set the time of day
`set_time <hh:mm>` SHALL move the world forward to the next moment with that time of day, never backward, and SHALL fail with a message for a malformed time.

#### Scenario: Skip to midnight
- **WHEN** it is 08:00 and the player runs `set_time 00:00`
- **THEN** it is 00:00 of the next day and the surface is dark

#### Scenario: Bad time
- **WHEN** the player runs `set_time 25:99`
- **THEN** nothing changes and the command fails with a message showing the expected format
