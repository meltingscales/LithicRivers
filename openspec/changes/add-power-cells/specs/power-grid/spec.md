# Spec Delta

## ADDED Requirements

### Requirement: Networks include batteries
A network's supply SHALL include what its batteries discharge, its surplus SHALL charge them, and the pole's info box SHALL show the stored energy.

#### Scenario: Info box
- **WHEN** the player opens a pole's info box on a network with a half-charged battery
- **THEN** it shows the network's stored energy alongside supply and demand
