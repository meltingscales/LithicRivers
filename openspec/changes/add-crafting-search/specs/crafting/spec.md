# Spec Delta

## ADDED Requirements

### Requirement: Recipe search
The Crafting tab SHALL filter its recipe list by a case-insensitive search over each recipe's name, its result item's name and description, and its ingredients' names, using the same search keys and behavior as the Item Dictionary. The list SHALL say how many recipes are shown, and crafting and the ingredient tree SHALL act on the selected shown recipe.

#### Scenario: Search for planks
- **WHEN** the player searches the Crafting tab for "plank"
- **THEN** the list shows the Wooden Plank recipe and the recipes that use wooden planks, and no others

#### Scenario: Craft from a filtered list
- **WHEN** the filter shows three recipes and the player selects the second and crafts
- **THEN** the second shown recipe is crafted

#### Scenario: Clear the search
- **WHEN** a filter is kept and the player presses Escape
- **THEN** every recipe is listed again
