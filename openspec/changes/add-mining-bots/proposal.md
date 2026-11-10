# Proposal

## Why

The Steam MVP's underground biome needs "dangerous mining bots" (`docs/STEAM-MVP.md`, core gameplay loop). The deep has lava and riverglass now (`add-molten-rivers`), but its only enemy is the nuclear slime, which also lives in the shallows. Rock walls keep the player safe from both. Mining bots are SapienCorp diggers still running their last dig order. They hear the player through the rock and drill straight toward them, so in the deep a wall is no longer a hiding place. They also carry what they dug, which makes hunting them worth it.

## What Changes

- **Mining bot:** a new enemy, only in the deep band.
  - **Hears through rock:** it notices the player within 10 tiles on its layer, with no line of sight needed.
  - **Drills:** each turn it steps toward the player. When the way is natural rock or a deposit (iron scrap, scrap electronics, plasteel scrap, riverglass), it drills that tile out instead, which takes the turn. It keeps the tile's mining drop. It never drills bedrock, lava, stairs, doors, built blocks or machines.
  - **Fights hard:** more health than a slime (150) and heavy drill hits. Combat starts when it is adjacent, as with dogs and slimes.
  - **Leaves a wreck with its haul:** it always carries some salvage (scrap electronics, sometimes iron scrap) plus whatever it drilled. It drops no embryos, since it is a machine.
- **Spawning:** a deep-band spawn rule with a small cap. Bots never spawn within 128 tiles of the SapienCorp bunker, so the escape from the starting bunker (z = -19) isn't a drilling ambush.
- **Cheats and scripts:** `spawn_enemy mining_bot` works like the other kinds.
- **Art:** a mining bot sprite following `docs/ART-DIRECTION.md`.

## Capabilities

### New Capabilities
- `mining-bots`: how mining bots notice, chase, drill, fight and drop.

### Modified Capabilities
- `enemy-spawning`: spawn rules can keep a kind away from the bunker; mining bots spawn in the deep.

## Impact

- **Core:** `components::MiningBot` (registered for saving), `EntityKind::MiningBot`, `EnemyKind::MiningBot`, `spawn_utils::world_spawn_mining_bot`, `crafting::MINING_BOT_LOOT`, `entity_ai::mining_bot_system` (scheduled before the combat trigger), the bot's combat stats, its wreck on death, a spawn rule with a new `away_from_bunker` field, and `game::BUNKER_POS`. `spawner::embryo` returns an `Option`, because a mining bot has no embryos.
- **Client:** the sprite, the combat panel name, `TestWorld::mining_bot`.
- **Depends on** `add-molten-rivers` for riverglass (one of the tiles a bot drills).
