# Spec Delta

## Purpose

How the desktop client connects to Steam: starting without Steam, the Steam overlay, cloud saves, and unlocking achievements.

## ADDED Requirements

### Requirement: The game runs without Steam
The client SHALL connect to Steam at launch when the build includes Steam support and Steam is running, and SHALL otherwise start and play exactly as it does without Steam, with no error shown to the player. Headless runs, MCP sessions, the terminal UI's tests and Android builds SHALL never connect to Steam.

#### Scenario: Steam isn't running
- **WHEN** a Steam-enabled build is launched while the Steam client is closed
- **THEN** the game reaches the title screen and plays normally, and the log notes that Steam is unavailable

#### Scenario: Build without Steam support
- **WHEN** the game is built without Steam support (development builds, tests, Android)
- **THEN** it neither needs the Steam API library to start nor tries to connect to Steam

### Requirement: Steam overlay
When launched from Steam, the game window SHALL show the Steam overlay on the overlay shortcut (Shift+Tab by default) on Linux, Windows and macOS.

#### Scenario: Opening the overlay
- **WHEN** the player launches the game from their Steam library and presses Shift+Tab in the game window
- **THEN** the Steam overlay opens over the game

### Requirement: Cloud saves
The world saves folder SHALL be synced by Steam Cloud for every platform, so a world saved on one machine appears in the world list on another machine signed in to the same Steam account. The save format and folder SHALL stay the same as without Steam.

#### Scenario: Continuing on another computer
- **WHEN** the player saves a world on one computer, quits, and launches the game from Steam on a second computer
- **THEN** the world is in the second computer's world list and loads where it was saved

### Requirement: Achievements unlock from play
The client SHALL unlock a Steam achievement when its condition is met during play, at most once per Steam account, and SHALL NOT unlock any achievement in a world that allows cheats. An achievement earned while Steam is unavailable is not retried later.

#### Scenario: Earned in a normal world
- **WHEN** the player meets an achievement's condition in a world without cheats while Steam is connected
- **THEN** Steam shows the achievement as unlocked

#### Scenario: Cheats allowed
- **WHEN** the player meets an achievement's condition in a world that allows cheats
- **THEN** no achievement is unlocked

### Requirement: Placeholder death achievement
The game SHALL have one placeholder achievement, "System Failure", unlocked when the robot dies for the first time.

#### Scenario: First death
- **WHEN** the robot is destroyed in a world without cheats
- **THEN** the "System Failure" achievement is unlocked

#### Scenario: Dying again
- **WHEN** the robot dies after "System Failure" is already unlocked
- **THEN** nothing new is unlocked and play continues as normal
