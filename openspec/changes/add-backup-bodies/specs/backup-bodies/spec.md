# Spec Delta

## Purpose

Prepare spare bodies in resurrection vats and wake up in one after dying, with a costly fallback when none is ready.

## ADDED Requirements

### Requirement: Resurrection vats hold spare bodies
A Robot Resurrection Vat SHALL hold one spare body assembled from crafted body parts, and SHALL be ready to resurrect into when its torso and fusion core slots are filled and, for powered tiers, it has power.

#### Scenario: A ready vat
- **WHEN** a powered standard vat holds a torso, a fusion core and two legs
- **THEN** its window shows it as ready, with the arms and head missing

### Requirement: Vat tiers
Vats SHALL come in three tiers: a crude starter vat that needs no power and wakes the robot in damaged crude parts, a mid-game standard vat that needs power and wakes it in working parts, and an end-game advanced vat that needs a lot of power and wakes it in enhanced parts.

#### Scenario: Crude vat
- **WHEN** the robot wakes in a crude vat's body
- **THEN** every part it was given is Damaged and the body has the default two arms and two legs

#### Scenario: Advanced vat
- **WHEN** a powered advanced vat holds advanced parts
- **THEN** the robot wakes with those parts Enhanced

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
