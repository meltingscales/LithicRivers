# Spec Delta

## MODIFIED Requirements

### Requirement: Search
The tab SHALL filter the list by a case-insensitive search over item names and descriptions, using the shared search keybinds (`search.SEARCH`, `search.DELETE_CHAR`, `search.END_SEARCH`) that the Crafting tab also uses.

#### Scenario: Search for fuel
- **WHEN** the player searches for "burn"
- **THEN** only items whose name or description contains "burn" are listed
