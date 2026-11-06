# Tasks

## 1. Core

- [x] 1.1 The teleporter block with a battery bank charging as a consumer; verify `cargo test` for charging, a full bank drawing nothing, and short networks
- [x] 1.2 Jumps: cost by distance, landing tile order, destination chunk generation, and refusal when short; verify tests for each, including across chunks and z-levels
- [x] 1.3 Default names and renaming; verify tests for unique names and a save/reload

## 2. Client and showcase

- [x] 2.1 The teleporter window (charge, destinations, cost, rename) and the arrival effect; verify snapshots and a headless jump test
- [x] 2.2 A recipe and a showcase example with two linked teleporters; verify the reachability and showcase coverage tests

## 3. Transit stations

- [x] 3.1 `TransitStation` blocks: unbreakable, dormant until first interaction, activation saved; verify tests for activation, dormant stations missing from lists, and a save/reload
- [x] 3.2 Destinations by source: free station-to-station jumps, stations in player teleporters' lists; verify tests for each source's list and cost
- [x] 3.3 Stations in the bunker's wake-up room, the factory, the android's site and the arm quest's facility; verify the golden `Game::new` entities, the structure lint, and a snapshot of the bunker room
- [x] 3.4 The activation message and the station window; verify a headless script that activates two stations and jumps between them, as a snapshot

## 4. Mobile tele-access device (after `add-power-cells`)

- [x] 4.1 The device item, recipe and description; verify the reachability and Item Dictionary tests
- [x] 4.2 Using it: the destination list, cost drawn from carried cells, "no charge" refusal; verify tests for a jump, an empty cell and a partly charged cell

## 5. Checks

- [ ] 5.1 Verify `just code-clippy`, `cargo test --workspace` and PR checks pass
- [ ] 5.2 User check that teleporting feels worth its cost, that stations are found and activated naturally on the way through quests, and that the device feels like a useful late convenience
