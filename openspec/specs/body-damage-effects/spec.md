# body-damage-effects Specification

## Purpose
Body damage has clear, readable consequences: legs slow walking, arms slow mining and soften melee, the head dims the light, and a damaged power source slows energy recovery.

## Requirements

### Requirement: Arms drive mining and melee
Mining and breaking SHALL take longer, and melee SHALL hit softer, for each damaged or missing arm. With no working arm, the robot SHALL be unable to mine or use melee. Leg damage SHALL not affect mining.

#### Scenario: One arm missing
- **WHEN** the left arm is missing and the right arm works
- **THEN** mining takes twice as long as with two working arms, and melee deals 40% less damage

#### Scenario: A damaged leg
- **WHEN** a leg is damaged and both arms work
- **THEN** walking takes twice as long and mining takes the normal time

### Requirement: Legs drive walking
Walking SHALL take twice as long with the weaker leg damaged, four times with one leg missing, and eight times with both missing, and SHALL never become impossible.

#### Scenario: Both legs missing
- **WHEN** both legs are missing
- **THEN** the robot can still move, at an eighth of normal speed

### Requirement: Head and power source
A damaged head SHALL shrink the light radius by one tile, and a missing head SHALL leave a radius of one. In combat the robot SHALL recover energy over time, and a damaged power source SHALL halve that recovery.

#### Scenario: Damaged power source in combat
- **WHEN** the power source is damaged and combat lasts 20 ticks
- **THEN** the robot recovers 5 energy instead of 10

#### Scenario: Damaged head with a torch
- **WHEN** the head is damaged and a torch is lit
- **THEN** the light radius is 7 tiles instead of 8

### Requirement: Effects are shown
The Body tab SHALL list every active damage effect in plain words, naming the part and what it changes.

#### Scenario: Starting body
- **WHEN** a new game's Body tab is opened (left arm missing, a damaged leg)
- **THEN** it lists the arm's mining and melee effect and the leg's walking effect
