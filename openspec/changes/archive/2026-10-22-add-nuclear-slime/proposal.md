# Proposal

## Why

Nuclear waste is meant to come only from ruined reactors (`add-ruined-reactors`). Those are rare and far away, so the thermoelectric tier can stall on a single unlucky seed. A creature that occasionally carries waste gives a second, slower source and a reason to fight. A glowing slime that has soaked in the old world's spills fits LithicRivers' backstory, and it's an enemy that plays differently from the fast feral dog.

## What Changes

- **Nuclear slime:** a new hostile creature that is slow, sturdy and glowing green.
  - It moves one tile every other turn toward the player once it notices them, and starts combat when adjacent, like a feral dog.
  - In combat it attacks less often than a dog but hits harder, and has more health.
- **Rare drop:** when a nuclear slime dies, it has a 5% chance to leave a corpse holding one nuclear waste. Otherwise it dissolves and leaves nothing.
  - Whether a given slime carries waste is rolled when it spawns, seeded by the world seed and its spawn position, so the same slime always drops the same thing.
- **Where they spawn:** underground chunks only, rarely, at a seeded chance per chunk (on top of feral dog spawns). Ruined reactors will add more around their ruins.
- **Nuclear waste item:** added here if `add-ruined-reactors` hasn't added it yet; whichever change lands first adds it. Creature loot counts as a raw source, so the crafting tree picks it up automatically.
- **Cheat:** `spawn_enemy nuclear_slime [x y z]`.

## Capabilities

### New Capabilities
- `nuclear-slime`: The nuclear slime's behavior, combat stats, spawning, and its rare nuclear waste drop.

### Modified Capabilities
- `cheat-console`: `spawn_enemy` supports `nuclear_slime` as well as `feral_dog`.

## Impact

- **Core:**
  - `EnemyKind::NuclearSlime` and `EntityKind::NuclearSlime`, plus a `NuclearSlime` marker component registered for saving.
  - Slime AI (slow chase) and its combat attacks.
  - A creature loot table with drop chances (`CreatureLoot { item, chance_percent, min, max }`) that replaces `FERAL_DOG_LOOT`'s plain ranges, so "rare" can be expressed.
  - Death handling becomes "any creature that carries items leaves a corpse", not only dogs.
  - Seeded underground spawning; `ItemKind::NuclearWaste` (if not already present).
- **Client:** slime and nuclear waste sprites; the slime in the combat panel's enemy list.
- **Tests:**
  - Drop rate over many seeded slimes.
  - Determinism, spawning only underground, and the corpse appearing only with waste.
  - The `spawn_enemy` cheat.
