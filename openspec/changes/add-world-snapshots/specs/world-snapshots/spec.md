# Spec Delta

## Purpose

Lets players roll a world back to recent points in its history without losing anything, and remember what happened between those points.

## ADDED Requirements

### Requirement: Automatic snapshots
While a saved world is being played, the client SHALL take a snapshot of the whole world every 5 game minutes of play and whenever the world is saved. A snapshot SHALL restore exactly the state it captured, using the same format as world saves.

#### Scenario: Snapshot after playing
- **WHEN** the player plays a saved world for 5 game minutes
- **THEN** a new snapshot exists in the world's folder

#### Scenario: Snapshot on save
- **WHEN** the player saves the world from the Menu tab
- **THEN** a snapshot of that moment exists

### Requirement: Branching timeline
Each snapshot SHALL record its parent: the snapshot the world had most recently been at. Rolling back SHALL first snapshot the present, then load the chosen snapshot, so later snapshots are its children and the abandoned future remains a separate branch.

#### Scenario: Roll back and continue
- **WHEN** snapshots A, B and C were taken in order, the player rolls back to A, and then a new snapshot D is taken
- **THEN** D's parent is A, and C (and the snapshot taken just before rolling back) are still in the timeline on their own branch

### Requirement: Action log
Each snapshot SHALL record what happened since its parent: the messages logged in that span with their ticks, and a one-line summary of game time, the player's position and health, and inventory changes.

#### Scenario: Read the log
- **WHEN** the player chopped a tree and fought a dog between two snapshots
- **THEN** the later snapshot's action log shows the chop and combat messages, and its summary shows the items gained

### Requirement: Timeline screen
The Timeline screen SHALL show the world's snapshots as a tree with branches indented, newest first, and mark the snapshot the world currently descends from. Selecting a snapshot SHALL show its summary and action log. Rolling back SHALL ask for confirmation.

#### Scenario: Cancel a rollback
- **WHEN** the player selects Roll back here and then cancels
- **THEN** the world is unchanged and no snapshot is taken

### Requirement: Pruning
After each new snapshot, a world SHALL keep only its 10 most recent snapshots plus the one it currently descends from, deleting the rest from disk. A snapshot whose parent was pruned SHALL still load and SHALL show its parent as pruned.

#### Scenario: Eleventh snapshot
- **WHEN** a world has 10 snapshots and an 11th is taken
- **THEN** the oldest snapshot not currently being descended from is deleted from disk, and 10 remain

### Requirement: Snapshots stay inside the world
Snapshots SHALL be stored inside the world's own folder, deleted together with the world, and never shared between worlds. Headless runs SHALL keep them in the run's temporary saves folder.

#### Scenario: Delete a world
- **WHEN** a world with snapshots is deleted from the world list
- **THEN** its snapshots are deleted too
