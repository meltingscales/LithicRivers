# Spec Delta

## Purpose

Guns the robot can use in combat, fed by an ammo supply chain whose brass casings come back to be reloaded.

## ADDED Requirements

### Requirement: Weapon slots
Each working arm SHALL hold at most one gun, equipped from the Body tab, and an arm that is missing SHALL hold none.

#### Scenario: Starting robot
- **WHEN** the starting robot (left arm missing) tries to equip two guns
- **THEN** only one is equipped, in the right arm

### Requirement: Shooting in combat
With at least one equipped gun that has matching ammo, the player SHALL have a Shoot move in combat that fires every equipped, loaded gun in the same turn. Each shot SHALL use one cartridge and hit by the same line-of-sight and distance rules as enemy shots, with its gun's own range, accuracy and damage.

#### Scenario: Shoot
- **WHEN** the player has a pipe gun equipped, 3 paper cartridges, and shoots a dog 3 tiles away
- **THEN** one cartridge is used and the shot hits or misses by the pipe gun's chance at 3 tiles

#### Scenario: Many arms, many shots
- **WHEN** a four-armed robot with four loaded pipe guns equipped uses Shoot once
- **THEN** four shots are fired that turn, each rolling its own hit

#### Scenario: No ammo
- **WHEN** the player has a pipe gun equipped and no paper cartridges
- **THEN** the Shoot move isn't offered

### Requirement: Casings return
Firing a brass cartridge SHALL leave one shell casing in the inventory.

#### Scenario: Reload loop
- **WHEN** the player fires 2 brass cartridges
- **THEN** they gain 2 shell casings

### Requirement: The ammo chain is craftable
Every gun and cartridge SHALL be craftable from raw sources the world provides, through gunpowder, paper, iron shot, and brass from nuggets to ingots to casings.

#### Scenario: Reachable
- **WHEN** the crafting reachability check runs
- **THEN** both guns and both cartridge kinds are reachable from raw sources
