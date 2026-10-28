# Spec Delta

## ADDED Requirements

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
