# Spec Delta

## Purpose

Markers so a wreck is never lost, and options to keep the inventory or the body through death.

## ADDED Requirements

### Requirement: Death markers
Dying SHALL add a marker at the wreck, shown on the Global Map and in the world view. The marker SHALL disappear when the wreck is looted empty, and the player SHALL be able to delete it after confirming, without affecting the wreck.

#### Scenario: Find the wreck
- **WHEN** the robot dies at (12, -40, -3) and respawns
- **THEN** the Global Map shows a death marker at (12, -40, -3)

#### Scenario: Delete a marker
- **WHEN** the player deletes a death marker and confirms
- **THEN** the marker is gone and the wreck still holds its items

### Requirement: Keep inventory
While `keepinventory` is on, dying SHALL keep everything the robot carried.

#### Scenario: Keep inventory
- **WHEN** `keepinventory on` is set and the robot dies carrying 3 logs
- **THEN** after respawning it still has the 3 logs, and the wreck is empty

### Requirement: Keep body
While `keepbody` is on, respawning SHALL give the robot the limbs it died with, every one of them at full integrity, and missing parts SHALL stay missing.

#### Scenario: Keep body
- **WHEN** `keepbody on` is set and the robot dies with a damaged leg and its left arm missing
- **THEN** after respawning the leg is at full integrity and the left arm is still missing
