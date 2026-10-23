# cheat-console Specification

## Purpose

Developer cheat commands for setting up game situations quickly, usable the same way from the in-game console, headless scripts and the MCP server.

## Requirements

### Requirement: Spawn enemy
`spawn_enemy <kind> [x y z]` SHALL spawn one enemy of the given kind at the given position, with the same components, health and seeded inventory as an enemy spawned by the game itself. Supported kinds: `feral_dog`, `nuclear_slime`.

#### Scenario: Spawn a feral dog
- **WHEN** the player is at (0, 0, 0) and runs `spawn_enemy feral_dog 3 0 0`
- **THEN** a feral dog with full health stands at (3, 0, 0) and the console logs where it was spawned

#### Scenario: Spawn a nuclear slime
- **WHEN** the player is at (0, 0, -20) and runs `spawn_enemy nuclear_slime 3 0 -20`
- **THEN** a nuclear slime with full health stands at (3, 0, -20) and the console logs where it was spawned

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

### Requirement: Cheats follow the world setting
The in-game cheat console SHALL be available only in worlds that allow cheats. In a world without cheats, the cheat-console keybind SHALL not open the console and SHALL show a message that cheats are disabled for this world. Headless `cheat` steps and the MCP `cheat` tool are test tooling and SHALL run in any world.

#### Scenario: Cheats disabled
- **WHEN** a world was created with Allow cheats off and the player presses the cheat-console keybind
- **THEN** the console does not open and the message log says cheats are disabled for this world

#### Scenario: Cheats enabled
- **WHEN** a world allows cheats and the player presses the cheat-console keybind
- **THEN** the console opens as before

### Requirement: Give crafting components
`givecraftingcomponents <item> [amount] [raw]` SHALL add to the player's inventory the ingredients needed to craft at least `amount` of the item (default 1), rounding up to whole crafts of its recipe. With `raw`, it SHALL instead give the raw materials at the bottom of the item's ingredient tree for that amount. It SHALL log what it gave. An unknown item, or an item with no recipe, SHALL be reported as an error and give nothing.

#### Scenario: Direct ingredients
- **WHEN** the player runs `givecraftingcomponents chest`
- **THEN** the inventory gains 8 wooden planks and 4 nails, and the log lists them

#### Scenario: Round up to whole crafts
- **WHEN** the player runs `givecraftingcomponents wooden_plank 4`
- **THEN** the inventory gains 2 logs, since one log makes 3 planks

#### Scenario: Raw materials
- **WHEN** the player runs `givecraftingcomponents chest 1 raw`
- **THEN** the inventory gains 3 logs and 1 iron scrap

#### Scenario: No recipe
- **WHEN** the player runs `givecraftingcomponents log`
- **THEN** nothing is given and the error says logs have no recipe

#### Scenario: Headless
- **WHEN** a script runs `cheat givecraftingcomponents chest` and then crafts a chest
- **THEN** the craft succeeds
