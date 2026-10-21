# Spec Delta

## ADDED Requirements

### Requirement: Give crafting components
`givecraftingcomponents <item> [amount] [raw]` SHALL add to the player's inventory the ingredients needed to craft at least `amount` of the item (default 1), rounding up to whole crafts of its recipe. With `raw`, it SHALL instead give the raw materials at the bottom of the item's ingredient tree for that amount. It SHALL log what it gave. An unknown item, or an item with no recipe, SHALL be reported as an error and give nothing.

#### Scenario: Direct ingredients
- **WHEN** the player runs `givecraftingcomponents chest`
- **THEN** the inventory gains 8 wooden planks and 4 nails, and the log lists them

#### Scenario: Round up to whole crafts
- **WHEN** the player runs `givecraftingcomponents wooden_plank 4`
- **THEN** the inventory gains 2 logs, since one log makes 3 planks

#### Scenario: Raw materials
- **WHEN** the player runs `givecraftingcomponents chest 1 raw`
- **THEN** the inventory gains 3 logs and 1 iron scrap

#### Scenario: No recipe
- **WHEN** the player runs `givecraftingcomponents log`
- **THEN** nothing is given and the error says logs have no recipe

#### Scenario: Headless
- **WHEN** a script runs `cheat givecraftingcomponents chest` and then crafts a chest
- **THEN** the craft succeeds
