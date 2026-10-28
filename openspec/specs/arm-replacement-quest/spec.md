# arm-replacement-quest Specification

## Purpose
The second quest: travel to the abandoned SapienCorp facility, find a salvaged arm in a guaranteed chest, and install it in place of the robot's missing left arm.

## Requirements

### Requirement: The quest
After the broken android's repair quest is complete, the android SHALL offer "A New Arm", whose objectives SHALL be to reach the facility, find the salvaged arm and install it, completed in order and shown in the Quests tab.

#### Scenario: Accept and finish
- **WHEN** the player accepts the quest, walks to the facility, takes the arm from its chest and installs it
- **THEN** every objective is complete and the quest shows as Completed

### Requirement: One guaranteed arm per world
The facility SHALL hold exactly one chest with one Salvaged SapienCorp Arm, placed once per world.

#### Scenario: Re-entering the facility
- **WHEN** the player leaves the facility's area and comes back
- **THEN** there is still only the one arm chest, and no second arm

### Requirement: Installing a replacement part
The player SHALL be able to install a matching replacement item into a missing body part from the Body tab, using up the item and making the part Functional. A part that isn't missing SHALL refuse an install.

#### Scenario: Install the arm
- **WHEN** the left arm is missing and the player installs a Salvaged SapienCorp Arm
- **THEN** the left arm is Functional at 70 integrity and the item is gone

#### Scenario: Nothing to replace
- **WHEN** the left arm already works and the player tries to install another arm
- **THEN** nothing changes and the message says the arm isn't missing
