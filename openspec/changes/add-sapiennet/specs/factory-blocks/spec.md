# Spec Delta

## ADDED Requirements

### Requirement: Blocks can wait on a condition
A factory block connected to a SapienNet network SHALL work only while its enable condition holds, and a block without a condition SHALL work as before.

#### Scenario: No condition
- **WHEN** a machine is connected to a network but has no condition
- **THEN** it works exactly as it would unconnected
