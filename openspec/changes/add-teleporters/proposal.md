# Proposal

## Why

The world is large and quests are far apart: the SapienCorp factory is a mile from the bunker. Once a player has bases in several places, walking between them is dead time. Buildable teleporters, like Minecraft's Waystones mod, let a player link their own bases, with a real energy cost so they belong to the mid and late game and tie into the power grid.

The user also wants SapienCorp's own teleporters at the major quest locations: each one needs to be used once to activate it, and works normally after that. The first is in the bunker room where the player wakes up. A "mobile tele-access device" that runs on power lets the player jump to these stations from anywhere.

## What Changes

- **Teleporter:** a mid-to-late-game factory block with a built-in battery bank that charges from the power grid it is connected to.
- **Naming:** a new teleporter gets a default name (such as "Teleporter 3") that the player can rename from its window.
- **Travelling:** interacting with a teleporter opens a list of every other teleporter in the world, by name, with its distance. Choosing one moves the player next to the destination.
  - Each jump costs a large amount of energy from the source teleporter's battery bank, rising with distance.
  - If the bank is short, nothing happens, and a message says how much more charge is needed.
- **Window:** shows the battery charge, the charge rate from the grid, and the cost to each destination.
- **Recipe:** later-tier materials (placeholder ingredients until those tiers exist).
- **SapienCorp transit stations:** teleporter stations built into the major quest locations: the bunker, the SapienCorp factory, the broken android's site and the arm quest's facility.
  - Each starts **dormant**. Interacting with it once runs an activation handshake ("Transit node registered"). After that it works normally.
  - Each station stands beside a SapienCorp radioisotope thermoelectric generator (RTG), an ordinary power pole and an Item Charger. The RTG charges the station's own battery bank, and a jump from a station costs the normal distance cost from that bank, which refills over time. A station lists only the other activated stations.
  - Stations, and their RTG and Item Charger, can't be broken or picked up.
- **RTG:** a generator with constant output and no fuel. Fixed at stations; a late-game recipe (placeholder ingredients) builds one for the player's own grid.
- **Item Charger:** a grid-powered block with cell slots that charges power cells straight from the grid. Fixed at stations, where it refills cells for the device; craftable mid-game.
- **The bunker's station:** it stands in the room where the player wakes up, so it is the first station they meet.
- **Mobile tele-access device:** a gear item that opens the destination list from anywhere, listing every activated station and every player-built teleporter.
  - It runs on carried power cells, by `add-power-cells`' gear rule: it draws from the fullest cell, and says "no charge" when every cell is empty.
  - A jump costs the same as from a player teleporter of the same distance.
- **Player-built teleporters list stations too:** activated stations appear in their destination lists.
- **Lore:** activation messages and the device's description hint at a SapienCorp transit network without explaining it (the slow-reveal rule in AGENTS.md).

## Capabilities

### New Capabilities
- `teleporters`: Teleporter blocks, battery banks charged from the grid, naming, the destination list, jump costs and the out-of-charge message; SapienCorp transit stations at quest locations, their activation and RTG-powered station-to-station travel; RTGs and Item Chargers; the mobile tele-access device.

### Modified Capabilities
<!-- None. The battery bank is an ordinary power consumer whose demand depends on its charge; the power-grid requirements hold unchanged. -->

## Impact

- **Core:** `FactoryKind::Teleporter` with `Teleporter { name, charge_j }`, charged as a power consumer up to its capacity, a jump function that checks the cost, moves the player and spends charge, and teleporters across chunks and z-levels.
- **Client:** the teleporter window (charge, destinations with distance and cost, rename) and a short arrival effect.
- **Core, stations:** `FactoryKind::TransitStation` with `TransitStation { name, activated }` (saved), a `teleporter_station` structure block, activation on first interaction, and destination lists that depend on the source (station, teleporter or device).
- **Structures:** a station in the bunker's wake-up room (`sapiencorp-bunker`), the factory, the android's site and the arm quest's facility.
- **Device:** a `MobileTeleAccessDevice` item and recipe, drawing its jump cost from carried power cells.
- **Client:** the same destination window for stations and the device, and the activation message.
- **Station power:** `FactoryKind::Rtg` and `FactoryKind::ItemCharger`, structure tiles for them and for a power pole, and a battery bank on each station.
- **Showcase:** a pair of linked teleporters example.
- **Depends on:** `add-power-cells` (merged), for the device and the Item Charger.
