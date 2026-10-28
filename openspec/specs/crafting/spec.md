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

### Requirement: Craftable-only filter
The Crafting tab SHALL have a toggle, by key and by clicking its hint, that lists only the recipes whose ingredients are all in the inventory. It SHALL combine with the search filter, update as the inventory changes, show its state in the list title, and be saved with the world.

#### Scenario: Starting kit
- **WHEN** a new game's Crafting tab has craftable-only turned on
- **THEN** only the recipes the starting kit can make are listed, and the title reads "Recipes (craftable only)"

#### Scenario: Crafting changes the list
- **WHEN** craftable-only is on and crafting uses up the last ingredients of another listed recipe
- **THEN** that recipe disappears from the list

#### Scenario: Nothing to craft
- **WHEN** craftable-only is on and nothing can be crafted
- **THEN** the list says "Nothing craftable yet" and names the key that shows all recipes

### Requirement: Recipe search
The Crafting tab SHALL filter its recipe list by a case-insensitive search over each recipe's name, its result item's name and description, and its ingredients' names, using the same search keys and behavior as the Item Dictionary. The list SHALL say how many recipes are shown, and crafting and the ingredient tree SHALL act on the selected shown recipe.

#### Scenario: Search for planks
- **WHEN** the player searches the Crafting tab for "plank"
- **THEN** the list shows the Wooden Plank recipe and every recipe that uses wooden planks, and each recipe shown mentions planks in its name, its result's name or description, or its ingredients

#### Scenario: Craft from a filtered list
- **WHEN** the filter shows three recipes and the player selects the second and crafts
- **THEN** the second shown recipe is crafted

#### Scenario: Clear the search
- **WHEN** a filter is kept and the player presses Escape
- **THEN** every recipe is listed again
