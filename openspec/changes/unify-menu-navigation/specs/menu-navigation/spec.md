# Spec Delta

## Purpose

Every menu, list and window answers to the same keys.

## ADDED Requirements

### Requirement: Both key sets move every menu
Every menu, list and window SHALL move its selection with both the numpad movement keys and the arrow keys (`ui.MENU_UP`, `MENU_DOWN`, `MENU_PREV`, `MENU_NEXT`), activate with `ui.MENU_ACTIVATE` and close with `ui.CLOSE_HELP_MENU`, read through keybinds rather than hard-coded keys.

#### Scenario: Body repair with arrow keys
- **WHEN** the body repair window is open and the player presses the Down arrow
- **THEN** the next repair recipe is selected, as with numpad 2

#### Scenario: Quests with the numpad
- **WHEN** the Quests tab lists two quests and the player presses numpad 2
- **THEN** the second quest is selected, as with the Down arrow

### Requirement: Open windows keep Left and Right
While a window is open, Left and Right SHALL move between its sides or do nothing, and SHALL never switch tabs behind it. On a tab's own list, Left and Right SHALL keep switching tabs.

#### Scenario: Left in the body repair window
- **WHEN** the body repair window is open and the player presses the Left arrow
- **THEN** the Body tab stays open behind the window
