# Spec Delta

## MODIFIED Requirements

### Requirement: Keep body
While `keepbody` is on, waking in the junker body SHALL give the robot the limbs it died with, every one of them at full integrity, and missing parts SHALL stay missing. Waking in a vat SHALL always give the vat's spare body.

#### Scenario: Keep body
- **WHEN** `keepbody on` is set and the robot dies with a damaged leg and its left arm missing, and wakes in the junker
- **THEN** the leg is at full integrity and the left arm is still missing
