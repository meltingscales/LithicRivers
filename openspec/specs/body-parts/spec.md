# body-parts Specification

## Purpose
Choose a body part in the Body tab, and install, swap or remove parts with the items in the inventory.

## Requirements

### Requirement: Choosing a part
The Body tab SHALL let the player select a body part with the menu keys or the mouse, and SHALL list the inventory's replacement items that fit the selected part.

#### Scenario: A salvaged arm fits the arms
- **WHEN** the inventory holds a Salvaged SapienCorp Arm and the player selects the left arm
- **THEN** the arm is listed as fitting it, and selecting a leg lists nothing

### Requirement: Install and swap
Installing a fitting item SHALL put it in the selected part. If that part isn't missing, the old part SHALL come out first, as removing it would.

#### Scenario: Swap a working arm
- **WHEN** the right arm is Functional and the player installs a Salvaged SapienCorp Arm into it
- **THEN** the right arm is Functional at 70 integrity and the inventory gains the old arm as a Salvaged SapienCorp Arm

### Requirement: Remove
Removing the selected part SHALL leave it missing and return a Functional or Enhanced part as its item, or a Damaged part as scrap. The torso and the power source SHALL NOT be removable.

#### Scenario: No free repairs
- **WHEN** the player removes a Damaged right leg
- **THEN** the leg is missing and the inventory gains scrap, not a part

#### Scenario: Locked torso
- **WHEN** the player tries to remove the torso
- **THEN** nothing changes and the message says the torso can't be removed
