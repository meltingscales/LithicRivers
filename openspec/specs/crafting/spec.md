# crafting Specification

## Purpose

Defines where materials come from and how they combine, and guarantees that every block the player can place can be crafted from materials found in the world.

## Requirements

### Requirement: Raw materials can be gathered
Mining SHALL yield materials: rock gives stone, iron scrap tiles give iron scrap, scrap electronics tiles give scrap electronics, and trees give logs and acorns. Creature loot (leather, meat) and these drops together SHALL be the raw sources for crafting.

#### Scenario: Mine iron scrap
- **WHEN** the player mines an iron scrap tile
- **THEN** the tile is removed and iron scrap items drop on it

#### Scenario: Mine rock
- **WHEN** the player mines a rock tile
- **THEN** the tile is removed and stone drops on it

### Requirement: Every placeable item is craftable
Every item that can be placed as a block SHALL have a recipe, and every ingredient in its recipe tree SHALL be either a raw source or itself craftable, with no cycles needed to obtain it. A test SHALL enforce this for all placeable items, including those added by later changes.

#### Scenario: New block without a recipe
- **WHEN** a developer adds a placeable item without a recipe, or with an ingredient that can't be obtained
- **THEN** `cargo test` fails and names the item and the missing ingredient

#### Scenario: Chest crafting tree
- **WHEN** a player starts with an empty inventory
- **THEN** a chest can be made by gathering logs and iron scrap and crafting planks, nails and then the chest

### Requirement: Intermediate recipes
Nails SHALL be craftable from iron scrap and string from leather, so items that today only exist in the starting inventory can be made.

#### Scenario: Craft nails
- **WHEN** the player has 1 iron scrap and crafts nails
- **THEN** the iron scrap is consumed and nails are added

### Requirement: Ingredient tree view
Selecting a recipe in the Crafting panel SHALL show its ingredients as a tree down to raw materials, with the quantity needed for one craft and how many of each the player has.

#### Scenario: View the chest tree
- **WHEN** the player selects the chest recipe
- **THEN** the panel shows planks (from logs) and nails (from iron scrap) with needed and owned counts
