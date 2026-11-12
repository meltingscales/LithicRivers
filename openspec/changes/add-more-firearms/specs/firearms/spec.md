# Spec Delta

## ADDED Requirements

### Requirement: Guns for every tier
The game SHALL offer, besides the pipe gun and rifle, a nail driver and a scattergun (early), a burst carbine and a slug thrower (mid), and an arc caster, a riverglass lance and an isotope launcher (late). Each SHALL be equipped into a weapon slot and fired by Shoot like the existing guns, and each SHALL be craftable from raw sources the world provides.

#### Scenario: Reachable
- **WHEN** the crafting reachability check runs
- **THEN** every gun and every kind of ammo is reachable from raw sources

### Requirement: Pellets
A gun that fires pellets SHALL roll a separate hit for each pellet of a round, each dealing its own damage.

#### Scenario: Scattergun up close
- **WHEN** the player fires one scatter shell at a dog 1 tile away
- **THEN** one shell is used and each pellet rolls its own hit

### Requirement: Bursts
A gun that fires bursts SHALL fire several rounds per Shoot, each using a round and rolling its own hit, and SHALL fire fewer when fewer rounds are carried.

#### Scenario: Burst carbine
- **WHEN** the player with 5 brass cartridges fires the burst carbine once
- **THEN** 3 cartridges are used, 3 hits are rolled and 3 shell casings are gained

#### Scenario: Short burst
- **WHEN** the player has 2 brass cartridges and fires the burst carbine
- **THEN** 2 rounds are fired

### Requirement: Stun
A gun with a stun chance SHALL stun the creature it hits by that chance, rolled from the world's seeded random number generator.

#### Scenario: Slug thrower
- **WHEN** many slugs hit creatures
- **THEN** about the slug thrower's stun chance of those hits leave the creature stunned

### Requirement: Charge ammo
A gun that uses charge SHALL draw a fixed charge per shot from the fullest carried power cell, and SHALL not be offered by Shoot when no carried cell holds enough.

#### Scenario: Arc caster
- **WHEN** the player with the arc caster fires once and carries one charged crude power cell
- **THEN** the cell loses one shot's charge

#### Scenario: Flat cells
- **WHEN** the player's only cell is empty
- **THEN** the arc caster isn't fired and Shoot isn't offered for it

### Requirement: Piercing
A piercing shot SHALL roll a hit against every enemy on its line from the robot out to its range, stopping at anything that blocks sight.

#### Scenario: Two in a row
- **WHEN** two dogs stand in a line east of the player and the riverglass lance fires at the nearer
- **THEN** a hit is rolled against each dog

### Requirement: Splash
A splash shot that hits SHALL also damage every other enemy within 1 tile of its target, and SHALL never damage the robot or change the world.

#### Scenario: Isotope launcher
- **WHEN** an isotope canister hits a dog with another dog beside it
- **THEN** both dogs take damage, and the robot and the tiles around them are unharmed
