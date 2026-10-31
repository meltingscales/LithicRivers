# Spec Delta

## Purpose

An agent that pathfinds and pursues goals on its own (reaching places, gathering and crafting items, building, and completing quest objectives) for tool-assisted speedruns and for tests that prove the game can be completed.

## ADDED Requirements

### Requirement: Goals
The autopilot SHALL pursue these goals: go to a position, get a number of an item, craft an item, build a blueprint or structure plan at a position, and complete the active quest's next objective. It SHALL plan any sub-goals a goal needs (ingredients, raw materials, tools to reach a place) on its own.

#### Scenario: Nested items
- **WHEN** a fresh world's robot is given "get 1 chest" with no planks or nails
- **THEN** the autopilot chops logs, makes planks, gets iron scrap, makes nails and crafts the chest, and the inventory holds a chest

#### Scenario: Unreachable goal
- **WHEN** a goal can't be planned (an item with no source, or a place no path reaches)
- **THEN** the autopilot stops and says which item or place it couldn't reach, leaving the game as it was when it gave up

### Requirement: Hunting and self-defense
For items only creatures carry, the autopilot SHALL find a creature that carries them, fight it with Melee, and loot its corpse. When attacked otherwise, it SHALL fight back with Melee, or use Escape when its power source or torso is badly damaged.

#### Scenario: Leather for string
- **WHEN** the robot needs 4 string and has 2, with feral dogs in the world
- **THEN** it kills a dog, loots its leather, and crafts string until it has 4

### Requirement: Long-range pathfinding
The autopilot SHALL find paths across loaded and not-yet-loaded chunks and between layers, using stairs and vertical conveyors, digging through blocks the robot can mine, and avoiding creatures it knows of.

#### Scenario: To the facility
- **WHEN** the robot at the bunker spawn is given "go to the SapienCorp facility entrance"
- **THEN** it walks there and stands within a tile of the entrance

### Requirement: Runs through player inputs
The autopilot SHALL be able to act only through the key presses a player would make, and such a run SHALL be exportable as a headless script that replays to the same state.

#### Scenario: Replay
- **WHEN** the input driver completes "get 1 chest" and the run is exported as a script
- **THEN** `just client-blind` on that script ends with a chest in the inventory and the same state

### Requirement: Player access
The `bot` cheat console commands SHALL start, show and stop the autopilot only in worlds that allow cheats or are TAS worlds. While it runs, the world view SHALL show its planned path and the Modes panel its goal and step, and any player key SHALL stop it.

#### Scenario: Normal world
- **WHEN** the player types `bot get log 5` in a world without cheats
- **THEN** nothing starts and the console says the autopilot needs cheats or a TAS world

#### Scenario: Taking over
- **WHEN** the autopilot is walking to a goal and the player presses a movement key
- **THEN** the autopilot stops and the robot takes that step

### Requirement: Determinism
Given the same world, seed and goal, the autopilot SHALL make the same plan and the same inputs, reading no wall-clock time.

#### Scenario: Two runs
- **WHEN** the same goal is run twice from the same save
- **THEN** both runs end on the same tick with the same state

### Requirement: The game can be completed
The test suite SHALL run the input driver from a fresh seed-12345 world and fail if it can't complete each milestone goal within its tick budget: the "A New Arm" quest, and getting a chest, a burner generator, an electric furnace and a crude resurrection vat.

#### Scenario: A change breaks progress
- **WHEN** a change removes the only source of iron scrap
- **THEN** the completability test fails, naming the goal and the item it couldn't get
