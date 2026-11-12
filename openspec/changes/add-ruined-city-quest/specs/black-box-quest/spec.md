# Spec Delta

## Purpose

"Black Box", the third quest: the repaired android sends the player to a SapienCorp datacenter in a ruined city to recover the reactor's last data, which reveals who pushed the reactor past its limits.

## ADDED Requirements

### Requirement: One datacenter per world, in a city
Every world SHALL have exactly one SapienCorp datacenter at a fixed place about half a mile from the bunker, always standing in a Ruined City, with a quest marker shown once the quest starts.

#### Scenario: Always there
- **WHEN** a new world is generated with any seed
- **THEN** the datacenter is at its fixed place and the ground around it is Ruined City

### Requirement: Offered after the arm
The repaired tutorial android SHALL offer "Black Box" once "A New Arm" is complete, and not before.

#### Scenario: Too early
- **WHEN** the player talks to the repaired android before installing the arm
- **THEN** "Black Box" is not offered

#### Scenario: Offered
- **WHEN** the player talks to the android after "A New Arm" is complete
- **THEN** the android offers "Black Box" and accepting it starts the quest and shows the datacenter's marker

### Requirement: The sealed door opens on power
A sealed door SHALL block the way while its power network is not powered, and SHALL let the player through while it is. The datacenter's server hall SHALL be behind one, on a network with an unfueled burner generator.

#### Scenario: Restoring power
- **WHEN** the player fuels the datacenter's generator
- **THEN** the sealed door opens and the quest's "Restore the power" step completes

#### Scenario: Power lost
- **WHEN** the generator runs out of fuel
- **THEN** the door closes again, but the quest step stays complete

### Requirement: The quest's steps
"Black Box" SHALL complete its steps in order: reach the datacenter, restore its power, take the data shard, and hand the shard to the android. Steps done early SHALL still count.

#### Scenario: Hand-in
- **WHEN** the player carrying the data shard talks to the android with the quest active
- **THEN** the shard is handed over, the quest completes, and the reward is given

### Requirement: The reward
Completing "Black Box" SHALL give a standard fusion core and one gun with ammo for it, the gun picked from a list of mid-tier guns by the world seed, so the same world always gives the same gun.

#### Scenario: Same world, same gun
- **WHEN** two worlds with the same seed complete the quest
- **THEN** both receive the same gun and the same amount of ammo

#### Scenario: Different worlds vary
- **WHEN** many different seeds complete the quest
- **THEN** more than one kind of gun is given

### Requirement: What the shard reveals
The data shard's description and the android's hand-in dialogue SHALL reveal that the reactor was pushed past its limits on purpose, under orders from SapienCorp's chief executive and board, for a project whose name stays redacted, run by a named project director, and SHALL point toward a next destination without explaining the project.

#### Scenario: Reading the shard
- **WHEN** the player looks at the data shard in the Item Dictionary
- **THEN** its description names the chief executive and the project director and keeps the project's name redacted
