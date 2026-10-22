# Spec Delta

## ADDED Requirements

### Requirement: Keep inventory and keep body switches
`keepinventory on|off` and `keepbody on|off` SHALL turn the matching death option on or off, save it with the world, and log the new state. Without an argument they SHALL report the current state.

#### Scenario: Toggle
- **WHEN** the player runs `keepinventory on`, saves and reloads the world
- **THEN** keep inventory is still on
