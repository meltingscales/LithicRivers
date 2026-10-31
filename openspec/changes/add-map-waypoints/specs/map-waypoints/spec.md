# Spec Delta

## Purpose

Player-made waypoints on the Global Map: creating them at the player's position, renaming and moving them, and deleting them, while quest markers can't be deleted.

## ADDED Requirements

### Requirement: Create a waypoint
On the Global Map, the add-waypoint key SHALL add a waypoint at the player's position named "Waypoint N", with N the lowest number no other waypoint uses, and SHALL select it.

#### Scenario: First waypoint
- **WHEN** the player stands at (4, -2, 0) with no waypoints and presses the add-waypoint key
- **THEN** a marker "Waypoint 1" at (4, -2, 0) is listed and selected

#### Scenario: Reusing a free number
- **WHEN** "Waypoint 1" was deleted and "Waypoint 2" remains, and the player adds a waypoint
- **THEN** the new waypoint is named "Waypoint 1"

### Requirement: Update a waypoint
With a waypoint selected, the rename key SHALL let the player type a new name, kept with Enter and dropped with Escape, and the move key SHALL move the waypoint to the player's position. Neither SHALL work on other markers.

#### Scenario: Rename
- **WHEN** the player selects "Waypoint 1", presses the rename key, erases the name, types "Iron field" and presses Enter
- **THEN** the waypoint is listed as "Iron field" at the same position

#### Scenario: Move
- **WHEN** the player walks to (10, 3, 0), selects a waypoint and presses the move key
- **THEN** the waypoint is at (10, 3, 0)

### Requirement: Delete waypoints but not quest markers
The delete key SHALL ask "Are you sure?" for a waypoint or a wreck marker and delete it on yes. On any other marker it SHALL say quest markers can't be deleted and change nothing.

#### Scenario: Delete a waypoint
- **WHEN** the player selects a waypoint, presses delete and answers yes
- **THEN** the waypoint is gone from the list and the map

#### Scenario: Quest marker
- **WHEN** the player selects the SapienCorp Factory marker and presses delete
- **THEN** the marker stays and the message says quest markers can't be deleted

### Requirement: Waypoints are saved
Waypoints SHALL be saved and loaded with the world, keeping their names and positions.

#### Scenario: Reload
- **WHEN** the player adds and renames a waypoint, saves and loads the world
- **THEN** the waypoint is listed with its new name and position
