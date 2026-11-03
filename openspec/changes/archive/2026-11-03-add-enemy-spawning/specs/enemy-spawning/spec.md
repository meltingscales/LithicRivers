# Spec Delta

## Purpose

Creatures keep appearing where they belong, within limits, and players can build farms around spawner blocks with fences.

## ADDED Requirements

### Requirement: Areas refill within limits
Creatures SHALL keep spawning by place-based rules, never in view, never within 16 tiles of the player, and never beyond each kind's cap within its radius.

#### Scenario: A cleared forest
- **WHEN** the player kills every dog near a forest and waits there out of view of it
- **THEN** dogs reappear over time, never more than the cap within its radius

#### Scenario: Never in view
- **WHEN** a spawn rule fires
- **THEN** no creature appears on a tile the player can currently see

### Requirement: Bases are safe from natural spawns
Natural spawns SHALL not happen near placed light sources or inside areas enclosed by fences and walls.

#### Scenario: Fenced base
- **WHEN** the player fences in an area
- **THEN** no creature spawns inside it

### Requirement: Spawner blocks
A spawner block SHALL spawn its creature nearby on a timer while the player is in range, up to its own cap. Wild spawners found in the world SHALL need nothing to run. Spawners SHALL be craftable, and breaking any spawner SHALL give a spawner item that can be placed again.

#### Scenario: Farm
- **WHEN** a dog spawner sits inside a fenced pen and the player waits nearby
- **THEN** dogs appear in the pen, never more than four from that spawner

### Requirement: Placed spawners need power, slurry and an embryo
A spawner the player places SHALL spawn only while it has grid power, meat slurry and an embryo. Each spawn SHALL use one meat slurry and one charge of the embryo; the embryo's creature kind SHALL set what spawns, and its grade SHALL set its charges (poor fewest, flawless most). Slurry and embryos SHALL be accepted from belts and extractors.

#### Scenario: Fed and powered
- **WHEN** a placed spawner has power, meat slurry and a poor feral dog embryo
- **THEN** feral dogs spawn on its timer, each using one slurry, until the embryo's charges run out

#### Scenario: Power cut
- **WHEN** a fed spawner loses power
- **THEN** it stops spawning until power returns, and keeps its slurry and embryo

#### Scenario: Out of slurry
- **WHEN** a powered spawner with an embryo runs out of meat slurry
- **THEN** it stops spawning and its embryo keeps its remaining charges

### Requirement: Creatures drop embryos
A killed creature SHALL sometimes drop one embryo of its own kind, poor most often and flawless most rarely, rolled from the seed like other loot, including creatures that came from spawners.

#### Scenario: Same seed, same drops
- **WHEN** the same creatures are killed on the same ticks in two runs of the same seed
- **THEN** they drop the same embryos

### Requirement: Fences and gates
Creatures SHALL not pass through fences or gates. The player SHALL not pass through fences, and SHALL walk through gates without opening them. Neither SHALL block sight or light.

#### Scenario: Behind a fence
- **WHEN** a dog chases the player and a fence is between them
- **THEN** the dog can't reach the player through the fence, and the player still sees it

#### Scenario: Through a gate
- **WHEN** the player walks into a gate in a fenced pen with a dog inside
- **THEN** the player steps onto the gate and into the pen, and the dog still can't leave through it
