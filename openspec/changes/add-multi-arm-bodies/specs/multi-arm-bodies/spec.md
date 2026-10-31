# Spec Delta

## Purpose

Bodies with four, six or eight arms, set by the torso, where every arm can be damaged, repaired and hold a weapon.

## ADDED Requirements

### Requirement: Layouts from the torso
A body's number of arm slots SHALL come from its torso: 2 for a standard torso, and 4, 6 or 8 for quad, hexa and octa torsos. Bodies from before this change SHALL load with 2 arms.

#### Scenario: A quad body
- **WHEN** a body has a quad torso
- **THEN** it has four arm slots, and the Body tab lists all four

#### Scenario: Old save
- **WHEN** a save from before this change is loaded
- **THEN** the robot has its two arms exactly as saved

### Requirement: Multi-arm bodies come from the advanced vat
The advanced resurrection vat SHALL accept quad, hexa and octa torsos, show an arm slot for each of the torso's arms, and wake the robot with that many arms.

#### Scenario: Advanced vat
- **WHEN** an advanced vat holds a hexa torso, a fusion core and six arms
- **THEN** the robot wakes from it with six arms

### Requirement: Every arm is a real part
Each extra arm SHALL take damage, be repaired, accept an installed replacement and hold one weapon, the same as the left and right arms.

#### Scenario: Damage reaches extra arms
- **WHEN** a six-armed robot takes many hits
- **THEN** extra arms are among the parts that get damaged, and the same seed gives the same sequence

### Requirement: Mining and melee use the best two arms
Mining and melee SHALL use the two best working arms, so extra arms don't make mining faster at full health.

#### Scenario: Four healthy arms
- **WHEN** a four-armed robot with every arm working mines a rock
- **THEN** it takes as long as for a two-armed robot with both arms working
