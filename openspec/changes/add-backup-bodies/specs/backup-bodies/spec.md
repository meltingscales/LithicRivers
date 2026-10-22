# Spec Delta

## Purpose

Prepare spare bodies in resurrection vats and wake up in one after dying, with a costly fallback when none is ready.

## ADDED Requirements

### Requirement: Resurrection vats hold spare bodies
A Robot Resurrection Vat SHALL hold one spare body assembled from crafted body parts, and SHALL be ready to resurrect into when its torso and fusion core slots are filled and it has power.

#### Scenario: A ready vat
- **WHEN** a powered vat holds a torso, a fusion core and two legs
- **THEN** its window shows it as ready, with the arms and head missing

### Requirement: Choosing where to wake
After dying, the player SHALL choose among every ready vat, each shown with its spare body's parts and distance, or the junker fallback. Waking in a vat SHALL use up its spare body.

#### Scenario: Two vats
- **WHEN** the robot dies with two ready vats in the world
- **THEN** the Resurrect screen lists both and the junker, and choosing one wakes the robot there in that vat's body, leaving the vat empty

### Requirement: The junker fallback
Without a ready vat, the robot SHALL wake at the SapienCorp HQ in a junker body with every part damaged.

#### Scenario: No vat
- **WHEN** the robot dies with no ready vat
- **THEN** it wakes at the HQ with every part damaged, and the message suggests building a vat

### Requirement: The respawn flyover
After a choice, the view SHALL fly from the wreck to the destination and zoom in before the robot wakes, any key SHALL skip it, and skipping SHALL not change the game's state.

#### Scenario: Skip
- **WHEN** the player presses a key during the flyover
- **THEN** the view is at the destination at zoom level 3, and the game is in the same state as without skipping
