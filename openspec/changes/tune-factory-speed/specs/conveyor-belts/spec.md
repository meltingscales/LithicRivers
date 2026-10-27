# Spec Delta

## MODIFIED Requirements

### Requirement: Belts carry items
A belt SHALL hold at most one item and SHALL move it one tile in its facing direction every two factory steps (under half a factory second), onto the next belt if that belt is empty.

#### Scenario: Item travels down a line
- **WHEN** a log is on the first of three east-facing belts in a row and one factory second passes
- **THEN** the log is on the third belt

#### Scenario: Corner
- **WHEN** an east-facing belt points into a south-facing belt
- **THEN** items move from the first onto the second and continue south
