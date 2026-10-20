# Spec Delta

## Purpose

Shows the keys that matter in the player's current situation on the World page, so players can play without memorizing controls or switching to the Help tab.

## ADDED Requirements

### Requirement: Context help panel
While the World page is shown outside combat and the panel is enabled, the client SHALL show a help panel below the inventory list. The panel SHALL list the keys relevant to the current situation, with key names taken from the keybind config.

#### Scenario: Walking
- **WHEN** the player is in movement mode with nothing to interact with nearby
- **THEN** the panel lists the move, mine, interact, look and build-mode keys

#### Scenario: Place mode
- **WHEN** the player switches to place mode
- **THEN** the panel lists the place-direction keys and the hotbar keys instead of the walking keys

#### Scenario: Look mode
- **WHEN** look mode is on
- **THEN** the panel lists the cursor keys and the key that leaves look mode

#### Scenario: Remapped key
- **WHEN** the config binds interact to a different key
- **THEN** the panel shows that key, not `f`

### Requirement: Nearby interactions
When something the player can interact with is adjacent, the panel SHALL show the interact key with what it would do. It SHALL agree with what pressing interact actually does.

#### Scenario: Next to a corpse with loot
- **WHEN** the player stands next to a corpse that still holds items
- **THEN** the panel shows the interact key with "loot corpse"

#### Scenario: Next to an empty corpse
- **WHEN** the player stands next to a corpse with no items and nothing else nearby
- **THEN** the panel shows no loot action

### Requirement: Toggle
The `ui.TOGGLE_CONTEXT_HELP` keybind (default `h`) SHALL show or hide the panel. When hidden, the sidebar SHALL still show a one-line hint naming the toggle key. The choice SHALL be stored in the config file (`ui.SHOW_CONTEXT_HELP`, default on) so it persists across launches; headless runs SHALL NOT write the config file.

#### Scenario: Hide and show
- **WHEN** the panel is shown and the player presses the toggle key
- **THEN** the panel is hidden, a hint names the toggle key, and pressing it again shows the panel

#### Scenario: Remembered
- **WHEN** the player hides the panel and restarts the game
- **THEN** the panel is still hidden

### Requirement: Not shown in combat
The context help panel SHALL NOT be shown while the combat panel is shown.

#### Scenario: Combat starts
- **WHEN** combat starts with the panel enabled
- **THEN** the World page shows the combat panel as before, without the help panel
