# item-dictionary Specification

## Purpose
An in-game reference for every item: what it looks like, what it's for, how to get it.

## Requirements

### Requirement: Every item is listed
The Item Dictionary tab SHALL list every item with its 1x sprite, its name and a short description, and SHALL scroll when the list is longer than the panel.

#### Scenario: New items appear
- **WHEN** a change adds an item kind with a sprite and description
- **THEN** it appears in the Item Dictionary without other changes

### Requirement: Search
The tab SHALL filter the list by a case-insensitive search over item names and descriptions.

#### Scenario: Search for fuel
- **WHEN** the player searches for "burn"
- **THEN** only items whose name or description contains "burn" are listed

### Requirement: Item details
The selected item SHALL show its 2x and 3x sprites and 12x8 art, the recipe that makes it, the recipes that use it, and its raw sources.

#### Scenario: Logs
- **WHEN** the player selects Log
- **THEN** the details show that trees drop it and list the recipes that use it, such as Wooden Plank and Power Pole

### Requirement: Readable cheat suggestions
The cheat console SHALL show autocomplete suggestions as a list with one suggestion per line, scrolling to keep the selected one visible, so no suggestion is cut off.

#### Scenario: Many items
- **WHEN** the player types `giveitem ` in the cheat console
- **THEN** every item name can be reached in the suggestion list, one per line
