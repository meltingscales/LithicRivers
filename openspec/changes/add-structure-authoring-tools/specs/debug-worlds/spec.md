# Spec Delta

## ADDED Requirements

### Requirement: Structure debug world
A `structure:<name>` debug world SHALL place the named structure at the origin of a flat world with fog of war off, and an unknown name SHALL fail with a message listing the structures.

#### Scenario: Unknown structure
- **WHEN** a script starts `world structure:castle`
- **THEN** the script fails with a message listing the available structures
