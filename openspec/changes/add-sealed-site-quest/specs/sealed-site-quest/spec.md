# Spec Delta

## Purpose

"Under the River of Fire", the fourth quest: SC-M4X7 sends the player down to Thale's Annex beneath a lava river, where Thale's journal reveals the aperture.

## ADDED Requirements

### Requirement: One annex per world, under a river
Every world SHALL have Thale's Annex at a fixed place in the deep band, beside a lava river, reachable from the surface by its access shaft, with a quest marker shown once the quest starts.

#### Scenario: Always there
- **WHEN** a new world is generated with any seed
- **THEN** the annex and its shaft are at their fixed place, and the shaft's stairs lead from the surface down to the annex

### Requirement: Offered after "Black Box"
The repaired android SHALL offer "Under the River of Fire" once "Black Box" is complete, and not before.

#### Scenario: Too early
- **WHEN** the player talks to the android before completing "Black Box"
- **THEN** the quest is not offered

#### Scenario: Offered
- **WHEN** the player talks to the android after "Black Box" is complete
- **THEN** the android offers the quest, and accepting it starts it and shows the annex's marker

### Requirement: The annex door needs the annex's own power
The annex's blast door SHALL be a sealed door on a network with no generator of its own, and the annex SHALL hold enough reactor casing and nuclear waste to craft and fuel one thermoelectric generator. Powering that network SHALL open the door.

#### Scenario: Building the way in
- **WHEN** the player mines the annex's outer reactor room, crafts a thermoelectric generator, loads it with nuclear waste and places it on the door's network
- **THEN** the blast door opens and the quest's "Power the annex door" step completes

#### Scenario: The station can't open it
- **WHEN** the Annex Station is woken and its RTG runs
- **THEN** the blast door stays sealed

### Requirement: The quest's steps
The quest SHALL complete its steps in order: reach the annex, power its door, take Thale's journal, and hand the journal to the android. Steps done early SHALL still count.

#### Scenario: Hand-in
- **WHEN** the player carrying Thale's journal talks to the android with the quest active
- **THEN** the journal is handed over, the quest completes, and the reward is given

### Requirement: The reward
Completing the quest SHALL give an advanced arm and a uranium chunk.

#### Scenario: Reward
- **WHEN** the journal is handed in
- **THEN** the player gains one advanced arm and one uranium chunk, in the inventory or on the ground if it is full

### Requirement: What the journal reveals
Thale's journal and the android's hand-in dialogue SHALL reveal that the aperture was a gate Thale built to reach "the far side," powered by the overloaded reactor, that it opened once for four seconds and something on the other side noticed, and that the maintenance frames were built to remember its shape. They SHALL NOT say what the player is for, and SHALL point toward where the aperture's parts were made.

#### Scenario: Reading the journal
- **WHEN** the player looks at Thale's journal in the Item Dictionary
- **THEN** its description names the aperture and Thale, and doesn't name the player's purpose
