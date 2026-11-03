## MODIFIED Requirements

### Requirement: Areas refill within limits
Creatures SHALL keep spawning by place-based rules, never in view, never within 16 tiles of the player, and never beyond each kind's cap within its radius. Each chunk SHALL spawn a creature naturally at most once; after that, only creatures that walk in and spawner blocks bring creatures there.

#### Scenario: A cleared forest
- **WHEN** the player kills every dog near a forest and waits there out of view of it
- **THEN** dogs reappear over time, never more than the cap within its radius

#### Scenario: Never in view
- **WHEN** a spawn rule fires
- **THEN** no creature appears on a tile the player can currently see

#### Scenario: A chunk refills once
- **WHEN** a creature has spawned naturally in a chunk, and the player clears it and waits nearby for many checks
- **THEN** no creature ever spawns naturally in that chunk again

#### Scenario: Spent chunks are remembered
- **WHEN** the player saves and reloads the world
- **THEN** the chunks that already spawned still never spawn naturally again
