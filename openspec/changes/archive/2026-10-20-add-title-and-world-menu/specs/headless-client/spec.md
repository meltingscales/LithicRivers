# Spec Delta

## MODIFIED Requirements

### Requirement: Run setup steps
`seed <u64>` and `size <cols>x<rows>` SHALL only appear before the first non-setup step and SHALL set the world seed and screen size. Without them the run SHALL use the configured `DEFAULT_SEED` and a 160x50 screen. `skip-splash` SHALL jump straight to the main game UI of a newly started world with the run's seed, cheats allowed, and the configured developer mode and tutorial defaults, bypassing the title menu.

#### Scenario: Seed after gameplay step
- **WHEN** `seed 5` appears after a `key` step
- **THEN** validation fails with status 1 and an error naming that line

#### Scenario: Skip splash
- **WHEN** a script starts with `skip-splash` followed by `screen`
- **THEN** the dumped screen shows the main game UI, not the logo or boot message

#### Scenario: Skip splash after the title screen
- **WHEN** a script waits until the title screen is shown and then runs `skip-splash`
- **THEN** a new world with the run's seed starts and the main game UI is shown

## ADDED Requirements

### Requirement: Isolated saves
Each headless run and each MCP session SHALL use its own new, empty saves folder that is removed when the run ends, so the world list starts empty and runs never read or modify the player's real worlds.

#### Scenario: Empty world list
- **WHEN** a headless script opens Play Game from the title screen
- **THEN** the world list shows only Create New World, regardless of worlds saved on the machine

#### Scenario: Worlds created in a run
- **WHEN** a script creates a world, saves and quits to the title, then opens Play Game
- **THEN** the list shows that one world, and the player's real saves folder is unchanged
