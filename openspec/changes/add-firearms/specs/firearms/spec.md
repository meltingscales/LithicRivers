# Spec Delta

## Purpose

Guns the robot can use in combat, fed by an ammo supply chain whose brass casings come back to be reloaded.

## ADDED Requirements

### Requirement: Shooting in combat
With a gun and matching ammo in the inventory, the player SHALL have a Shoot move in combat that uses one cartridge and hits by the same line-of-sight and distance rules as enemy shots, with the gun's own range, accuracy and damage.

#### Scenario: Shoot
- **WHEN** the player has a pipe gun and 3 paper cartridges and shoots a dog 3 tiles away
- **THEN** one cartridge is used and the shot hits or misses by the pipe gun's chance at 3 tiles

#### Scenario: No ammo
- **WHEN** the player has a pipe gun and no paper cartridges
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
