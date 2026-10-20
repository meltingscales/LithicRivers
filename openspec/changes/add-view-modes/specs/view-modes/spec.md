# Spec Delta

## Purpose

Lets the player focus the world view on one system at a time (power, enemies, logistics) without changing how the game plays.

## ADDED Requirements

### Requirement: Switching view modes
The player SHALL cycle through Normal, Power, Enemies and Logistics views with next/previous view-mode keybinds. The current mode SHALL be shown in the MODES panel. Changing the view SHALL NOT change the game state or advance time.

#### Scenario: Cycle views
- **WHEN** the player presses the next-view-mode key from Normal
- **THEN** the view shows Power mode, the MODES panel says Power, and the game tick is unchanged

#### Scenario: Wrap around
- **WHEN** the player presses the next-view-mode key in Logistics
- **THEN** the view returns to Normal

### Requirement: Focused views dim everything else
In every mode except Normal, terrain and things not part of that mode SHALL be drawn dimmed, and the mode's subject SHALL be drawn at full brightness. Fog of war still applies.

#### Scenario: Terrain dimmed
- **WHEN** the player switches to Enemies view
- **THEN** grass, trees, belts and machines are drawn dimmed and feral dogs are highlighted

### Requirement: Power view
Power view SHALL show poles, the outline of each pole's supply area, wires, and generators with their fuel status, and SHALL color each power consumer by state: powered, partly powered, unpowered, or idle. Wires SHALL blink every second as in Normal view.

#### Scenario: Starved machine
- **WHEN** a machine is on a network that can supply half its demand
- **THEN** in Power view it is drawn in the partly-powered color

#### Scenario: Out of range
- **WHEN** a machine is outside every pole's supply area
- **THEN** in Power view it is drawn in the unpowered color, outside any supply outline

### Requirement: Enemies view
Enemies view SHALL highlight every hostile creature within view and draw neutral creatures and NPCs dimmed.

#### Scenario: Find the dogs
- **WHEN** two feral dogs and a sheep are on screen in Enemies view
- **THEN** both dogs are highlighted and the sheep is dimmed

### Requirement: Logistics view
Logistics view SHALL show belts with their direction and carried items, chests and machines with a fill indicator for their inventories, and machines marked as working, idle, or stuck (output full or missing input).

#### Scenario: Stuck harvester
- **WHEN** an auto-harvester's output is full and nothing accepts its items
- **THEN** in Logistics view it is marked as stuck and its fill indicator shows full

#### Scenario: Belt contents
- **WHEN** a belt line carries logs
- **THEN** in Logistics view each belt shows its direction and the log glyph where a log is
