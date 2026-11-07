# Spec Delta

## Purpose

Mining bots are SapienCorp diggers in the deep band that hear the player through rock and drill toward them.

## ADDED Requirements

### Requirement: Hears through rock
A mining bot SHALL notice the player within 10 tiles on its layer whether or not it can see them, and SHALL do nothing while the player is farther away or on another layer.

#### Scenario: Behind a wall
- **WHEN** a mining bot is 5 tiles east of the player with solid rock between them and the player waits
- **THEN** the bot starts coming toward the player

#### Scenario: Out of earshot
- **WHEN** a mining bot is 12 tiles from the player and the player waits several turns
- **THEN** the bot stays where it is

### Requirement: Drills toward the player
Each turn, a mining bot that has noticed the player SHALL step one tile toward them. When no step toward the player is open and the tile in the way is natural rock or a deposit, it SHALL drill that tile into air instead and keep the tile's mining drop. It SHALL NOT drill bedrock, lava, stairs, doors, built blocks or factory blocks.

#### Scenario: Through rock
- **WHEN** a mining bot is 4 tiles east of the player with a wall of rock between them and the player waits
- **THEN** the rock in its way turns to air, the bot reaches the player, and combat starts

#### Scenario: Keeps what it drills
- **WHEN** a mining bot drills an iron scrap tile
- **THEN** the tile is air and the bot carries the iron scrap the tile drops

#### Scenario: Built walls hold
- **WHEN** a mining bot is behind a wall of stone bricks
- **THEN** the bricks stay and the bot doesn't get through

### Requirement: Combat and wreck
A mining bot SHALL have more health than a nuclear slime. When it dies, it SHALL leave a wreck holding its salvage and everything it drilled, and SHALL NOT drop an embryo.

#### Scenario: Tougher than a slime
- **WHEN** a mining bot and a nuclear slime are spawned
- **THEN** the bot has more health

#### Scenario: Wreck
- **WHEN** a mining bot that drilled an iron scrap tile is killed
- **THEN** a wreck remains on its tile holding scrap electronics and that iron scrap
