# Design

## Context

- `Body { parts: HashMap<BodyPartType, BodyPart> }` covers the head, torso, power source, arms and legs, with part states. The Body tab shows it.
- `add-blueprints` pastes through a `BuildCost` policy hook.

## Goals / Non-Goals

**Goals:** blueprints become earned and paced; tiers give a reason to upgrade; deterministic.

**Non-Goals:** other module kinds (combat, mining) beyond leaving room for them; nanobots repairing machines.

## Decisions

### Modules in the body
- The torso has 4 module slots. A module is an item installed or removed in the Body tab; a damaged or missing torso disables them.
- Module power: Mk1 = 1, Mk2 = 3, Mk3 = 8. Build speed is total power blocks per game second, and range is 6 + 4 × the highest installed tier, in tiles.

### Build queue
- A paste checks and pays the full cost up front (as in `add-blueprints`), then queues its blocks in blueprint order.
- Each factory step adds module power ÷ steps per second to a progress counter, and places the next queued block in range when it reaches 1.
- Each block costs the robot's power source a fixed amount of energy; with too little, the queue pauses.
- The queue is saved. Cancelling refunds the blocks not yet built.

### Cheat
- `nanobots_free` sets a `Resources` flag: pastes skip the module check and place everything at once.
- It's refused when the world's Allow cheats setting is off, like other console commands.

## Risks / Trade-offs

- [A queue the player forgets about] → An indicator shows queued blocks; cancelling refunds.
- [Balance across tiers] → The numbers are constants; tune with the factory stats change.
