# Spec Delta

## ADDED Requirements

### Requirement: Factories run at a third of game speed
Factory block rates and power figures SHALL be measured in factory seconds, and one factory second SHALL last 3 game seconds (300 ticks), so a block works through one smelt or action in several player steps. A factory second SHALL be 5 factory steps.

#### Scenario: Smelting takes several steps
- **WHEN** a fueled crude furnace holds a log and the player walks six tiles (200 ticks each)
- **THEN** the log has been smelted into biochar, and not before the 12th game second
