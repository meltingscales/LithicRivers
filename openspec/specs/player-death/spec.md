# player-death Specification

## Purpose
What happens when the robot dies: a clear screen, the old body left as a wreck with its items, and a backup body to continue in.

## Requirements

### Requirement: System-failure screen
When the robot dies, the game SHALL show a system-failure screen naming what destroyed it, offering to respawn at the bunker or quit to the title, and SHALL take no other game input until one is chosen.

#### Scenario: Dying in a fight
- **WHEN** a feral dog destroys the robot's power source
- **THEN** the system-failure screen names the destroyed part and offers both choices

#### Scenario: Quitting while dead
- **WHEN** the player quits to the title from the screen and plays the world again
- **THEN** the system-failure screen is shown again

### Requirement: The wreck keeps the inventory and the body
The robot's old body SHALL stay where it died as a wreck holding everything the robot carried, lootable like a corpse, and keeping the old body's parts.

#### Scenario: Recover items
- **WHEN** the robot dies carrying 3 logs, respawns, walks back and loots the wreck
- **THEN** the player has the 3 logs and the wreck is empty

### Requirement: Respawning
Respawning SHALL move the player to the respawn point (the bunker spawn of a new game) with the default body and the starting kit, and SHALL leave the world, factories and quests as they were.

#### Scenario: Respawn
- **WHEN** the player chooses to respawn at the bunker
- **THEN** the player stands at the bunker spawn, alive, with the default body and kit, and the wreck is still where they died
