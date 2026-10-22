# mouse-menus Specification

## Purpose
Mouse support for every menu, list, form and window, matching what the keyboard does.

## Requirements

### Requirement: Clicking a menu row
Clicking a row in a menu or list SHALL select it and do what pressing Enter on it does: title screen choices, the world list and its actions, Choose Interaction, dialogue choices, the filter picker, the hotbar block picker, combat moves and cheat console suggestions. In the crafting and Item Dictionary lists, a click SHALL select the row, and clicking the selected recipe again SHALL craft it.

#### Scenario: Title screen
- **WHEN** the player clicks "Watch Demo" on the title screen
- **THEN** the demo starts, as if they had selected it and pressed Enter

#### Scenario: Choose Interaction
- **WHEN** the Choose Interaction menu lists a chest and a generator and the player clicks the generator's row
- **THEN** the generator's window opens

### Requirement: Clicking in item windows
In chest, corpse, generator, machine and furnace windows, clicking a row SHALL focus its panel, select the row and move one item as Enter does; right-clicking a row SHALL move the whole stack, through the same rules (a machine still refuses what it doesn't take). Clicking a furnace slot SHALL select it.

#### Scenario: Store a stack of logs
- **WHEN** the player right-clicks "6 x Log" in Your Items with a chest open
- **THEN** all 6 logs move into the chest

### Requirement: Form fields
Clicking a Create New World row SHALL focus it; clicking a toggle SHALL flip it, clicking World type SHALL cycle it, and clicking Create or Cancel SHALL press it.

#### Scenario: Toggle cheats
- **WHEN** the player clicks "Allow cheats: OFF"
- **THEN** it reads "Allow cheats: ON"

### Requirement: Clicking outside closes
A click outside an open window or menu, and on no other clickable row, SHALL close it as Escape does.

#### Scenario: Close a chest window
- **WHEN** a chest window is open and the player clicks the map outside it
- **THEN** the window closes and nothing is built or broken

### Requirement: Hover in menus
The row under the mouse SHALL be highlighted in menus, lists and windows.

#### Scenario: Hover a title choice
- **WHEN** the mouse rests over "Options" on the title screen
- **THEN** that row is highlighted

### Requirement: Rows record their own click areas
Every widget that draws clickable rows SHALL record each visible row's screen area and target while drawing, and clicks SHALL be resolved only from those recorded areas, so drawing and clicking always agree.

#### Scenario: A scrolled list
- **WHEN** the Item Dictionary list is scrolled so its 10th item is the first visible row and the player clicks that row
- **THEN** the 10th item is selected
