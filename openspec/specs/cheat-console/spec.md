# cheat-console Specification

## Purpose

Developer cheat commands for setting up game situations quickly, usable the same way from the in-game console, headless scripts and the MCP server.

## Requirements

### Requirement: Spawn enemy
`spawn_enemy <kind> [x y z]` SHALL spawn one enemy of the given kind at the given position, with the same components, health and seeded inventory as an enemy spawned by the game itself. Supported kinds: `feral_dog`.

#### Scenario: Spawn a feral dog
- **WHEN** the player is at (0, 0, 0) and runs `spawn_enemy feral_dog 3 0 0`
- **THEN** a feral dog with full health stands at (3, 0, 0) and the console logs where it was spawned

#### Scenario: Unknown kind
- **WHEN** a player runs `spawn_enemy dragon`
- **THEN** nothing is spawned and the command fails with a message listing the supported kinds

### Requirement: Default spawn position
Without coordinates, `spawn_enemy` SHALL spawn two tiles east of the player on the player's level: close by, but never on the player's tile. The enemy then behaves normally; a feral dog chases the player and combat starts once it is adjacent.

#### Scenario: Spawn without coordinates
- **WHEN** the player is at (5, 5, 0) and runs `spawn_enemy feral_dog`
- **THEN** the dog appears at (7, 5, 0) and combat is not yet active

#### Scenario: The dog closes in
- **WHEN** the player then waits one turn
- **THEN** the dog moves next to the player and combat starts, with the combat panel shown

### Requirement: Player-relative coordinates
Commands that take a position (`spawn_enemy`, `tp`) SHALL accept each axis as an absolute integer or as `~` followed by an optional signed offset from the player's position on that axis. `~` alone means no offset. Absolute and relative axes may be mixed.

#### Scenario: Relative spawn
- **WHEN** the player is at (10, 10, -19) and runs `spawn_enemy feral_dog ~-2 ~1 ~`
- **THEN** the dog appears at (8, 11, -19)

#### Scenario: Relative teleport
- **WHEN** the player is at (10, 10, -19) and runs `tp ~5 ~ 0`
- **THEN** the player is moved to (15, 10, 0)

#### Scenario: Malformed coordinate
- **WHEN** a player runs `tp ~x 0 0` or gives only two coordinates
- **THEN** nothing moves and the command fails with a usage message

### Requirement: Spawn location must be free
`spawn_enemy` SHALL refuse to spawn onto a tile that is not passable or that already holds an entity that blocks movement, including the player.

#### Scenario: Occupied tile
- **WHEN** a player runs `spawn_enemy feral_dog ~ ~ ~`
- **THEN** nothing is spawned and the command fails saying the tile is occupied

### Requirement: Spawned enemies are deterministic
An enemy spawned at the same position in a world with the same seed SHALL be identical, including its inventory.

#### Scenario: Same spawn twice
- **WHEN** two games with the same seed each run `spawn_enemy feral_dog 3 0 0`
- **THEN** both dogs have the same health and inventory

### Requirement: Failures are reported, never fatal
A cheat command that fails SHALL log its message in the console's error color and leave the game running. Headless script `cheat` steps and the MCP `cheat` tool SHALL report the same failure as an error.

#### Scenario: Failure in the interactive console
- **WHEN** a player enters `spawn_enemy dragon` in the in-game console
- **THEN** a red error message appears in the message log and the game keeps running

#### Scenario: Failure in a script
- **WHEN** a headless script runs `cheat spawn_enemy dragon`
- **THEN** the script stops with exit status 1 and an error naming the line and the failure
