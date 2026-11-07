# Spec Delta

## Purpose

Steam achievements: which exist, when each unlocks, and that worlds with cheats unlock none.

## ADDED Requirements

### Requirement: The achievement list
The game SHALL have these achievements: System Failure (`ACH_FIRST_DEATH`, lose your body), Daylight (`ACH_SURFACE`, reach the surface), Self-Defense (`ACH_FIRST_KILL`, destroy an enemy), Spare Parts (`ACH_REPAIR_ANDROID`, complete Repair the Broken Android), Two Hands (`ACH_REPLACE_ARM`, complete the arm quest) and Transit (`ACH_FIRST_JUMP`, teleport for the first time).

#### Scenario: Reaching the surface
- **WHEN** the player of a world without cheats stands on the surface
- **THEN** Daylight is unlocked

#### Scenario: First kill
- **WHEN** the player of a world without cheats destroys an enemy
- **THEN** Self-Defense is unlocked

#### Scenario: Quests
- **WHEN** the player of a world without cheats completes the arm quest
- **THEN** Two Hands is unlocked

#### Scenario: First jump
- **WHEN** the player of a world without cheats teleports
- **THEN** Transit is unlocked

### Requirement: Unlocked once, never with cheats
Each achievement SHALL be sent to Steam at most once per session, and none SHALL unlock in a world that allows cheats.

#### Scenario: Staying on the surface
- **WHEN** the player stays on the surface for many frames
- **THEN** Daylight is unlocked once

#### Scenario: Cheats
- **WHEN** the player of a world with cheats reaches the surface and destroys an enemy
- **THEN** nothing is unlocked

### Requirement: Player stats
The world SHALL count the enemies the player has destroyed and the teleports they have taken, and SHALL keep the counts in its save.

#### Scenario: Counting and saving
- **WHEN** the player destroys two enemies, teleports once, saves and reloads
- **THEN** the counts are two enemies and one teleport
