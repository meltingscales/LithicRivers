# Spec Delta

## Purpose

Enemies that shoot from a distance with limited ammo, where distance and cover change the odds.

## ADDED Requirements

### Requirement: Ranged attacks
A creature with a ranged attack SHALL be able to start combat and shoot when the robot is within its range on the same layer and in its line of sight. Walls SHALL block line of sight, and the chance to hit SHALL fall as distance grows.

#### Scenario: Behind cover
- **WHEN** a wall is between the android and the robot
- **THEN** the android can't shoot the robot

#### Scenario: Farther is safer
- **WHEN** the android shoots from 6 tiles instead of 2
- **THEN** its chance to hit is lower

### Requirement: Limited ammo
A ranged creature SHALL use one ammo per shot and, when out of ammo, SHALL close in and fight in melee.

#### Scenario: Out of ammo
- **WHEN** an android with 1 shot left has fired it
- **THEN** it moves toward the robot instead of shooting

### Requirement: The rusty military android
The rusty military android SHALL keep its distance while it has ammo, have more health than a feral dog, and leave its leftover ammo and scrap when destroyed.

#### Scenario: Keeps away
- **WHEN** the robot walks up to an android with ammo
- **THEN** the android steps back to stay 4 to 6 tiles away while it can
